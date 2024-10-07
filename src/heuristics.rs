use wasm_bindgen::prelude::wasm_bindgen;
use sqlparser::ast::Visitor;
use crate::binding::StmtAnalyzer;

#[wasm_bindgen]
pub struct AnalyzerReport
{
        issue_type: IssueType,
        severity: Severity,
        message: String,
}

impl AnalyzerReport
{
    pub fn create_report(issue: &dyn Issue) -> AnalyzerReport
    {
        AnalyzerReport { issue_type: issue.get_type(), severity: issue.get_severity(), message: issue.get_message()  }
    }
}

#[wasm_bindgen]
pub enum Severity
{
    WARNING = "WARNING",
    ERROR = "ERROR",
}

#[wasm_bindgen]
pub enum IssueType
{
        INCOMPLETEJOIN = "IncompleteJoin",
        REDUDANTDISTINCT = "RedudantDistinct",
        
}

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

        self.redudant_distinct();

        Ok(vec!())
    }

    fn redudant_distinct(&self) -> Vec<RedudantDistinctIssue>
    { 
        vec!()
    }



}
