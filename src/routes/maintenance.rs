use crate::database::maintenance::list_maintenance_status;
use actix_web::{get, web, HttpResponse};
use crate::models::errors::ApiResponse;
use crate::state::AppState;


#[get("/list")]
pub async fn get_maintenance_list(state: web::Data<AppState>) -> ApiResponse {
    // Get the maintenance status from the database
    let maintenance_status = list_maintenance_status(&state.pg_pool).await?;

    Ok(HttpResponse::Ok().json(maintenance_status))
}
