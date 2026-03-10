use actix_web::{post, web, HttpResponse, Responder};

use crate::models::*;
use crate::ssh_connection_pool::ssh_connection_pool::SshCommandRunner;

#[utoipa::path(
    post,
    path = "/command/exec",
    request_body = CommandRequest,
    responses(
        (status = 200, description = "Command execution result", body = CommandResponse)
    )
)]
#[post("/command/exec")]
pub async fn exec_command(runner: web::Data<SshCommandRunner>,
                          request: web::Json<CommandRequest>)-> impl Responder
{
    let sudo: bool = request.sudo.unwrap_or(false);
    match runner.execCommand(&request.command, sudo).await {
        Ok(result) => {
            HttpResponse::Ok().json(
                CommandResponse {
                    stdout: result.stdout,
                    stderr: result.stderr,
                    exit_code: result.exitCode,
                }
            )
        }
        Err(err) => {
            HttpResponse::InternalServerError().body(err.to_string())
        }
    }
}