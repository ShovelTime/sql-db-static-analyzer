
pub trait Issue : ToString
{
}

pub struct IncompleteJoinIssue
{
}

impl Issue for IncompleteJoinIssue
{
    
}

impl ToString for IncompleteJoinIssue
{
    fn to_string(&self) -> String {
        todo!();
    }
}
