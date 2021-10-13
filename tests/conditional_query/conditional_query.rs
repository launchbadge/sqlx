#[test]
fn simple() {
    let result = sqlx::conditional_query_as!(
        testing X,
        "A" "B" ?id + 1
    );
    assert_eq!(result, ("X", "A B $1", vec!["id + 1"]))
}

#[test]
fn single_if() {
    let limit = Some(1);
    let result = sqlx::conditional_query_as!(
        testing X,
        "A" "B" ?id + 1
        if let Some(limit) = limit { "C" ?limit }
        "D"
    );
    assert_eq!(result, ("X", "A B $1 C $2 D", vec!["id + 1", "limit"]))
}

#[test]
fn if_else() {
    let value = true;
    let result = sqlx::conditional_query_as!(
        testing X,
        "A" if value { "B" } else { "C" } "D"
    );
    assert_eq!(result, ("X", "A B D", vec![]))
}

#[test]
fn if_else_2() {
    let value = false;
    let result = sqlx::conditional_query_as!(
        testing X,
        "A" if value { "B" } else { "C" } "D"
    );
    assert_eq!(result, ("X", "A C D", vec![]))
}

#[test]
fn single_if_2() {
    let limit: Option<usize> = None;
    let result = sqlx::conditional_query_as!(
        testing X,
        "A" "B" ?id + 1
        if let Some(limit) = limit { "C" ?limit }
        "D"
    );
    assert_eq!(result, ("X", "A B $1 D", vec!["id + 1"]))
}

#[test]
fn single_match() {
    enum Y {
        C,
        D,
    }
    let value = Y::D;

    let result = sqlx::conditional_query_as!(
        testing X,
        "A"
        "B" ?id + 1
        match value {
            Y::C => "C",
            Y::D => "D",
        }
    );
    assert_eq!(result, ("X", "A B $1 D", vec!["id + 1"]))
}

#[test]
fn nested_if() {
    let result = sqlx::conditional_query_as!(
        testing X,
        "A"
        "B" ?id + 1
        if false {
            if true {
                if true {
                    "C"
                } else {
                    "D"
                }
            }
        } else if true {
            if true {
                "D"
            }
        }
    );
    assert_eq!(result, ("X", "A B $1 D", vec!["id + 1"]))
}

#[test]
fn empty() {
    let result = sqlx::conditional_query_as!(testing A, "");
    assert_eq!(result, ("A", "", vec![]))
}

#[test]
fn empty_2() {
    let result = sqlx::conditional_query_as!(testing A,);
    assert_eq!(result, ("A", "", vec![]))
}
#[test]
fn empty_3() {
    let result = sqlx::conditional_query_as!(testing A, if false { "X" });
    assert_eq!(result, ("A", "", vec![]))
}
