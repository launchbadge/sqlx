use {
    std::env,
    std::pin::Pin,
    anyhow::Context,
    log::*,
    actix_web::*,
    futures::{
        prelude::*,
        task,
        task::Poll,
        stream::BoxStream,
    },
    serde::{Deserialize, Serialize},
    sqlx::{
        prelude::*,
        postgres::*,
        query_as,
        pool::PoolConnection,
    },
    ouroboros::self_referencing,
};

#[actix_web::main]
async fn main() -> anyhow::Result<()>
{
    dotenv::dotenv().ok();
    env_logger::init();
    let db_url =
        env::var("DATABASE_URL")
        .context("DATABASE_URL")?;
    let pool: PgPool =
        PgPool::connect(&db_url)
        .await
        .context(db_url)?;
    let addr =
        env::var("SOCKETADDR")
        .unwrap_or("127.0.0.1:8080".to_string());
    info!("this web server is listening at http://{}", &addr);
    HttpServer::new(move || {
        actix_web::App::new()
            .wrap(middleware::Logger::default())
            .app_data(web::Data::new(pool.clone()))
            .service(widgets)
    })
        .bind(&addr)
        .context(addr)?
        .run()
        .await
        .context("while starting actix web server")?;
    Ok(())
}

#[post("/widgets")]
async fn widgets(
    web::Json(params): web::Json<WidgetParams>,
    pool:              web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error>
{
    log::info!("/widets {:?}", &params);
    let conn =
        pool
        .acquire()
        .await
        .map_err(|e|actix_error(&e.to_string()))?;
    let sql_stmt =
        WidgetStream::sql_stmt(&params);
    Ok(HttpResponse::Ok()
       .content_type("application/json")
       .streaming({
           WidgetStream::build(
               params,
               sql_stmt,
               conn,
           )
       }))
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Widget {
    pub id:            i64,
    pub serial:        i64,
    pub name:          String,
    pub description:   String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetParams {
    pub begin:     i64,
    pub end:       i64,
    pub where_:    Option<String>,
}

#[self_referencing]
pub struct WidgetStream {
    pub started:   bool,
    pub finished:  bool,
    pub params:    Box<WidgetParams>,
    pub sql_stmt:  Box<String>,
    pub conn:      Box<PoolConnection<Postgres>>,
    #[borrows(sql_stmt, mut conn)]
    pub widgetstream: BoxStream<'this, Result<Widget, sqlx::Error>>,
}
impl<'a> WidgetStream {
    fn build(
        params:    WidgetParams,
        sql_stmt:  String,
        conn:      PoolConnection<Postgres>,
    ) -> Self
    {
        WidgetStreamBuilder {
            started:   false,
            finished:  false,
            params:    Box::new(params.clone()),
            sql_stmt:  Box::new(sql_stmt),
            conn:      Box::new(conn),
            widgetstream_builder:
            | sql_stmt:   &String,
            conn:   &mut PoolConnection<Postgres> |
            {
                query_as::<Postgres, Widget>(sql_stmt.as_str())
                    .bind(params.begin)
                    .bind(params.end)
                    .fetch(conn)
                    .boxed()
            }
        }.build()
    }
    fn sql_stmt(
        params:    &WidgetParams,
    ) -> String
    {
        let mut sql_stmt = "
SELECT * FROM widgets
WHERE id BETWEEN $1 AND $2 ".to_string();            //  WHERE  (id BETWEEN ? AND ?)
        if let Some(where_) = &params.where_ {
            sql_stmt.push_str(&format!("
AND ( {} ) ", where_));
        }
        sql_stmt
    }
}

impl<'a> Stream for WidgetStream {
    type Item = Result<web::Bytes, actix_http::Error>;
    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>
    ) -> Poll<Option<Self::Item>>
    {
        if *self.borrow_finished() {
            return Poll::Ready(None);
        }
        let mut buf = Vec::<u8>::with_capacity(4096);
        loop {
            match self.with_widgetstream_mut(
                |strm|strm.as_mut().poll_next(cx)
            )
            {
                Poll::Ready(Some(Ok(rec))) => {
                    if *self.borrow_started() {
                        buf.push(b',');
                    } else {
                        buf.push(b'[');
                        self.with_started_mut(
                            |s| *s = true
                        );
                    }
                    serde_json::to_writer(&mut buf, &rec).ok();
                    if buf.len() < 2048 {
                        continue;
                    } else {
                        return Poll::Ready(Some(Ok(
                            web::Bytes::copy_from_slice(&buf)
                        )));
                    }
                },
                Poll::Ready(Some(Err(e))) => {
                    error!("{:?}", e);
                    continue;
                },
                Poll::Ready(None) => {
                    self.with_finished_mut(
                        |s| *s = true
                    );
                    buf.extend_from_slice("]".as_bytes());
                    return Poll::Ready(Some(Ok(
                        web::Bytes::copy_from_slice(&buf)
                    )));
                },
                Poll::Pending => {
                    return if buf.is_empty() {
                        Poll::Pending
                    } else {
                        Poll::Ready(Some(Ok(
                            web::Bytes::copy_from_slice(&buf)
                        )))
                    };
                }
            }
        }
    }
}

fn actix_error(s: &str) -> actix_web::Error
{
    actix_web::Error::from(
        actix_http::ResponseBuilder::new(
            http::StatusCode::BAD_REQUEST
        ).json(
            serde_json::json!({"error": s})
        )
    )
}
