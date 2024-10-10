use std::{collections::HashSet, ops::ControlFlow};

use wasm_bindgen::prelude::wasm_bindgen;
use sqlparser::ast::{visit_statements, Expr, GroupByExpr, Select, SelectItem::{ExprWithAlias, QualifiedWildcard, UnnamedExpr, Wildcard}, Statement, TableFactor};
use crate::binding::StmtAnalyzer;

#[wasm_bindgen]
pub struct AnalyzerReport
{
        #[wasm_bindgen(getter_with_clone)]
        pub issue_type: IssueType,
        #[wasm_bindgen(getter_with_clone)]
        pub severity: Severity,
        #[wasm_bindgen(getter_with_clone)]
        pub message: String,
}

impl AnalyzerReport
{
    pub fn create_report(issue: &dyn Issue) -> AnalyzerReport
    {
        AnalyzerReport { issue_type: issue.get_type(), severity: issue.get_severity(), message: issue.get_message()  }
    }
}

#[wasm_bindgen]
#[derive(Copy, Clone, Debug)]
pub enum Severity
{
    WARNING,
    ERROR ,
}

#[wasm_bindgen]
#[derive(Copy, Clone, Debug)]
pub enum IssueType
{
        INCOMPLETEJOIN,
        REDUDANTDISTINCT,
        
}


//I would Rather be able to expose this to Javascript instead of the AnalyzerReport struct, but
//wasm_bindgen has no idea on how to export that yet.
pub trait Issue
{
    fn get_type(&self) -> IssueType;
    fn get_severity(&self) -> Severity;
    fn get_message(&self) -> String;
    fn get_report(&self) -> AnalyzerReport;
}




pub struct IncompleteJoinIssue
{
    table1: String,
    table2: String,
}

impl Issue for IncompleteJoinIssue
{
    fn get_severity(&self) -> Severity {
        Severity::WARNING
    }

    fn get_type(&self) -> IssueType {
        IssueType::INCOMPLETEJOIN
    }
    

    fn get_report(&self) -> AnalyzerReport {
        AnalyzerReport::create_report(self)
    }


    fn get_message(&self) -> String {
        format!("Tables {} and {} are not joined per the FK-PK relationship", self.table1, self.table2)
    }
}

pub struct RedudantDistinctIssue
{
}

impl Issue for RedudantDistinctIssue
{
    fn get_severity(&self) -> Severity {
        Severity::WARNING
    }

    fn get_type(&self) -> IssueType {
        IssueType::REDUDANTDISTINCT
    }

    fn get_message(&self) -> String {
        "DISTINCT has no effect when a correct GROUB BY clause is present".to_string()
    }

    fn get_report(&self) -> AnalyzerReport {
        AnalyzerReport::create_report(self)
    }
}

#[wasm_bindgen]
impl StmtAnalyzer
{

    #[wasm_bindgen(js_name = analyzeAST)]
    pub fn analyze_ast(&self) -> Result<Vec<AnalyzerReport>, String>
    {
        if self.tree.is_empty() {return Err("No AST has been parsed yet".to_string())};

        let mut issues = Vec::<Box<dyn Issue>>::new();

        issues.append(&mut self.redudant_distinct());

        Ok(vec!())
    }

    fn redudant_distinct(&self) -> Vec<Box<dyn Issue>>
    {
        let mut issues : Vec<Box<dyn Issue>> = Vec::new();
        //Recursively walk through the AST, and execute the closure on every statement-type token
        //In this case we want to catch SELECT.
        visit_statements(&self.tree, |stmt| {
            match stmt {
                 Statement::Query(query) => {
                    //Try to Simplify the SELECT statement.
                    match query.body.as_select() {
                        Some(select) => {
                            //Cant be a redudant distinct if there is no distinct or groupbys
                            match select.distinct{
                                Some(_) => {
                                    //check for GroupBy
                                    match &select.group_by{
                                        GroupByExpr::All(_) => todo!("Snowflake, DuckDB and ClickHouse syntax not supported yet"),
                                        GroupByExpr::Expressions(expressions, ..) =>
                                        {
                                            if !expressions.is_empty()
                                            {
                                                let select_values = self.extract_select_value(select);
                                                let group_by_values = self.extract_group_by_values(expressions);
                                                if select_values.difference(&group_by_values).collect::<Vec<&String>>().is_empty() {
                                                    issues.push(Box::new(RedudantDistinctIssue{}));

                                                }

                                                                                                
                                                

                                            }

                                        }
                                    }
                                    
                                },
                                None => (),
                            }
                            
                        },

                        //The hard way then.
                        None => {}
                        }
                    
                }
                _ => ()
            }
            ControlFlow::<()>::Continue(())
        });
        issues
    }

    ///Returns a list of column names, or their aliases used within this particular statement
    ///Will attempt to expand wildcards according to the tables listed in the FROM clause.
    ///TODO: Subquery handling.
    fn extract_select_value<'a>(&'a self, query: &'a Select) -> HashSet<String>
    {
        let items = &query.projection;
        let mut selection : HashSet<String> = HashSet::with_capacity(items.len());
        for item in items{
            match item
            {
                UnnamedExpr(expr) => {
                    match expr {
                        Expr::Identifier(ident) => 
                        {
                            selection.insert(ident.value.clone());
                        }
                        Expr::CompoundIdentifier(ident) =>{
                            let column_ident : String = ident.iter().map(|ident| ident.value.clone()).collect::<Vec<String>>().join("."); 
                            if column_ident == ""  {continue;}
                            
                            selection.insert(column_ident);
                        }

                        //higher match arms should catch this.
                        Expr::Wildcard | Expr::QualifiedWildcard(_) => unreachable!("Wildcard found in UnnamedExpr, yell at ShovelTime"),
                        
                        //TODO: Subquery resolving?
                        _ => ()
                    
                    }
                },

                ExprWithAlias { alias , .. } => {
                    selection.insert(alias.value.clone());

                },

                QualifiedWildcard(obj, ..) =>
                {
                    let Some(table_name) = obj.0.last() else{unreachable!("Impossible state from AST parser, Empty Qualified Wildcard!")};
                    if table_name.value == "*" {todo!("parser includes wildcard as the objectname, yell at ShovelTime to fix ASAP.")};
                    let table = self.database_columns.get(&table_name.value);
                    match table{
                        Some(res) => {
                            selection.extend(res.columns.clone());
                        }
                        None => ()
                    }                    
                },

                Wildcard(_) => {
                    // we will have to infer the column names from the table descriptions.
                    // TODO: resolve subqueries in FROM, would probably entail offloading into a
                    // seperate function
                    let from_tables = &query.from;
                    for table in from_tables
                    {
                        match &table.relation {
                            TableFactor::Table { name, ..} => {
                                let Some(table_name) = name.0.first() else {unreachable!("Impossible State from AST parser, no table name!")};
                                let table = self.database_columns.get(&table_name.value);
                                match table{
                                    Some(res) => {
                                        selection.extend(res.columns.clone());
                                    }
                                    None => ()
                                }
                            },

                            _ => todo!(),
                        }
                           
                    }
                },
            }
        }

        selection

    }

    fn extract_group_by_values(&self, expressions: &Vec<Expr>) -> HashSet<String>
    {
        let mut selection = HashSet::with_capacity(expressions.len());
        for expr in expressions
        {
            match expr
            {
                Expr::Identifier(ident) => 
                {
                    selection.insert(ident.value.clone());
                }
                Expr::CompoundIdentifier(ident) =>{
                    let column_ident : String = ident.iter().map(|ident| ident.value.clone()).collect::<Vec<String>>().join("."); 
                    if column_ident == ""  {continue;}
                    
                    selection.insert(column_ident);
                }

                //que? how would that even compile.
                Expr::Wildcard | Expr::QualifiedWildcard(..) => (), // Needs further investigation.
                
                //Nothing else should be allowed, but worth looking into.
                _ => ()
                    
                    
            }
        }
        selection
    }

    
        

}


