
use std::collections::HashMap;

use sqlparser::ast::Statement;
use sqlparser::dialect::{dialect_from_str, Dialect, SQLiteDialect};
use sqlparser::parser::Parser;
use sqlparser::parser::ParserError::{ParserError, RecursionLimitExceeded, TokenizerError};
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::{JsValue, UnwrapThrowExt};



/// Wrapper function for dialect determination, done so in the event that we change SQLparser, or
/// want to determine dialect in another way, inline is suggested to the compiler in the cases
/// where the function is small. 
#[inline]
fn determine_dialect_from_str(string: &str) -> Option<Box<dyn Dialect>>
{
    dialect_from_str(string)
}

#[wasm_bindgen]
pub struct DatabaseColumn
{
        #[wasm_bindgen(getter_with_clone)]
        pub name: String,
        #[wasm_bindgen(getter_with_clone)]
        pub columns: Vec<String>
}

#[wasm_bindgen]
impl DatabaseColumn
{

    #[wasm_bindgen(constructor)]
    pub fn new(name : String, columns: Vec<String>) -> DatabaseColumn
    {
        DatabaseColumn { name, columns }
    }

}

#[wasm_bindgen]
pub struct JoinRules
{
        #[wasm_bindgen(getter_with_clone)]
        pub column1: String,
        #[wasm_bindgen(getter_with_clone)]
        pub key1: Vec<String>,
        #[wasm_bindgen(getter_with_clone)]
        pub column2: String,
        #[wasm_bindgen(getter_with_clone)]
        pub key2: Vec<String>,
}

#[wasm_bindgen]
impl JoinRules
{
    pub fn new(column1 : String, key1: Vec<String>, column2: String, key2: Vec<String>) -> JoinRules
    {
        JoinRules { column1, key1, column2, key2 }
    }
}

#[wasm_bindgen]
pub struct StmtAnalyzer{
        dialect_str: String,
        dialect: Box<dyn Dialect>,
        stmt: Option<String>,

        

        #[wasm_bindgen(skip)]
        pub tree: Vec<Statement>,
        #[wasm_bindgen(skip)]
        pub database_columns : HashMap<String, DatabaseColumn>,
        #[wasm_bindgen(skip)]
        pub join_rules: Vec<JoinRules>,

}

#[wasm_bindgen]
impl StmtAnalyzer
{

    /// Construct Analyzer for a statement, given information about the current database.
    /// the parser will try to determine the dialect from the given string, if it is unable to, or
    /// the string input is null, it
    /// will instead default to SQLite.
    /// valid options are :
    /// - generic
    /// - mysql
    /// - postgresql
    /// - postgres
    /// - hive
    /// - sqlite
    /// - snowflake
    /// - redshift
    /// - mssql
    /// - clickhouse
    /// - bigquery
    /// - ansi
    /// - duckdb
    /// - databricks
    #[wasm_bindgen(constructor)] 
    pub fn new(database_columns: Vec<DatabaseColumn>, join_rules: Vec<JoinRules>, dialect: Option<String>) -> StmtAnalyzer
    {

        let dialect_str = dialect.unwrap_or("sqlite".to_string());
        let mut column_map = HashMap::with_capacity(database_columns.len());
        for column in database_columns
        {
            column_map.insert(column.name.clone(), column);
        }
        let db_dialect = determine_dialect_from_str(dialect_str.as_str()).unwrap_or(Box::new(SQLiteDialect {}));
        StmtAnalyzer {
            database_columns: column_map,
            join_rules,
            dialect_str,
            dialect: db_dialect,
            stmt: None,
            tree: Vec::new()
        }
    }

    #[wasm_bindgen(getter = getDialect)]
    pub fn dialect(&self) -> String
    {
        self.dialect_str.clone()
    }

    #[wasm_bindgen(getter = getStatement)]
    pub fn stmt(&self) -> Option<String>
    {
        self.stmt.clone()
    }

    #[wasm_bindgen(getter = getAST)]
    pub fn tree(&self) -> JsValue
    {
       serde_wasm_bindgen::to_value(&self.tree).unwrap_throw() 
    }

    /// parses SQL statement and builds AST, returns nothing if successful, throws an Expection on
    /// failure with a string description.
    #[wasm_bindgen(js_name = parseStatement)]
    pub fn parse_stmt(&mut self, stmt: &str) -> Result<(), String>
    {
        let parsed = Parser::parse_sql(self.dialect.as_ref(), stmt);
        match parsed
        {
            Ok(ast) => {
                self.tree = ast;
                self.stmt = Some(stmt.to_string());
                Ok(())
            }
            Err(err) => {
                match err {
                    TokenizerError(str) | ParserError(str) => Err(str),
                    RecursionLimitExceeded => Err("Parser hit recursion limit".to_string()),
                }
            }
        }
    }
}
