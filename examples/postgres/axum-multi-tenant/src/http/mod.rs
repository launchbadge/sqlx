use accounts::AccountsManager;
use color_eyre::eyre;
use sqlx::PgPool;

pub async fn run(pool: PgPool, accounts: AccountsManager) -> eyre::Result<()> {
    axum::serve
}
