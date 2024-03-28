use dotenv::dotenv;
use mysql_async::prelude::*;
pub use mysql_async::*;

pub async fn get_pool() -> Pool {
    dotenv().ok();
    let url = std::env::var("DATABASE_URL").expect("not url db url found");

    let opts = Opts::from_url(&url).unwrap();
    let builder = OptsBuilder::from_opts(opts);
    // The connection pool will have a min of 5 and max of 10 connections.
    let constraints = PoolConstraints::new(5, 10).unwrap();
    let pool_opts = PoolOpts::default().with_constraints(constraints);

    Pool::new(builder.pool_opts(pool_opts))
}

pub async fn list_issues(
    pool: &Pool,
    page: usize,
    page_size: usize,
) -> Result<Vec<(String, String, String, String)>> {
    let mut conn = pool.get_conn().await?;
    let offset = (page - 1) * page_size;
    let issues: Vec<(String, String, String, String)> = conn
        .query_map(
            format!(
                "SELECT issue_id, project_id, issue_title, issue_description FROM issues_master ORDER BY issue_id LIMIT {} OFFSET {}",
                page_size, offset
            ),
            |(issue_id, project_id, issue_title, issue_description): (String, String, String, String)| {
                (issue_id, project_id, issue_title, issue_description)
            },
        )
        .await?;

    Ok(issues)
}
