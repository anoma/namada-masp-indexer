pub mod sql {
    use diesel::expression::functions::define_sql_function;
    use diesel::sql_types::Integer;

    define_sql_function!(fn abs(x: Integer) -> Integer);
}
