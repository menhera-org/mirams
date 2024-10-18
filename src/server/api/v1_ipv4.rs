//!
//! Endpoints for the IPv4 API
//! - `GET /api/v1/ipv4/assignment_space` - List all assignment spaces
//! - `POST /api/v1/ipv4/assignment_space` - Create a new assignment space
//! - `GET /api/v1/ipv4/assignment_space/:space_id` - Get an assignment space by ID
//! - `PUT /api/v1/ipv4/assignment_space/:space_id` - Update metadata for an assignment space by ID
//! - `DELETE /api/v1/ipv4/assignment_space/:space_id` - Delete an assignment space by ID
//! - `GET /api/v1/ipv4/assignment_space/:space_id/pool` - List all pools in an assignment space
//! - `POST /api/v1/ipv4/assignment_space/:space_id/pool` - Create a new pool in an assignment space
//! - `GET /api/v1/ipv4/assignment_space/:space_id/pool/:pool_id` - Get a pool by ID
//! - `PUT /api/v1/ipv4/assignment_space/:space_id/pool/:pool_id` - Update metadata for a pool by ID
//! - `DELETE /api/v1/ipv4/assignment_space/:space_id/pool/:pool_id` - Delete a pool by ID
//! - `GET /api/v1/ipv4/assignment_space/:space_id/pool/:pool_id/assignment` - List all assignments in a pool
//! - `POST /api/v1/ipv4/assignment_space/:space_id/pool/:pool_id/assignment` - Create a new assignment in a pool
//! - `GET /api/v1/ipv4/assignment_space/:space_id/pool/:pool_id/assignment/:assignment_id` - Get an assignment by ID
//! - `PUT /api/v1/ipv4/assignment_space/:space_id/pool/:pool_id/assignment/:assignment_id` - Update metadata for an assignment by ID
//! - `DELETE /api/v1/ipv4/assignment_space/:space_id/pool/:pool_id/assignment/:assignment_id` - Delete an assignment by ID
//! 
//! GET endpoints accept unauthenticated requests if object visibility is set to `ObjectVisibility::Public`.



use crate::store::DbConnection;
use crate::server::Server;
use crate::types::ObjectVisibility;
use super::AuthHandler;
use super::fallback_handler;
use super::build_json_response;
use super::User;
use super::ApiResponseVariant;
use super::ApiResponse;
use super::MetadataUpdateRequest;
use super::run_blocking_task;

use crate::ipv4::{
    AssignmentSpaceIpv4,
    AssignmentPoolIpv4,
    AssignmentIpv4,
};

use axum::Router;
use axum::body::Body;
use axum::routing::{get, post, put, delete};
use axum::extract::Extension as ExtensionExtractor;
use axum::extract::Json as JsonExtractor;
use axum::extract::Path as PathExtractor;

use http::Response;


async fn api_v1_ipv4_assignment_space_list<T>(ext: Option<ExtensionExtractor<Server<T>>>, user: Option<ExtensionExtractor<User>>) -> Response<Body>
where
    T: DbConnection + Clone + Send + Sync + 'static,
{
    if let Some(ext) = ext {
        let store = ext.0.store();
        let res = match run_blocking_task(store.clone(), |store| store.ipv4_assignments().get_spaces()).await {
            Ok(mut spaces) => {
                if let None = user {
                    spaces.retain(|space| {
                        space.space_visibility == ObjectVisibility::Public
                    });
                }
                let res = ApiResponse {
                    error: None,
                    result: Some(ApiResponseVariant::Ipv4AssignmentSpaces(spaces)),
                };
                build_json_response(res, 200)
            },
            Err(e) => {
                let res = ApiResponse {
                    error: Some(format!("Error listing assignment spaces: {}", e)),
                    result: None,
                };
                build_json_response(res, 400)
            },
        };
        return res;
    } else {
        let res = ApiResponse {
            error: Some("Internal Server Error".to_string()),
            result: None,
        };
        return build_json_response(res, 500);
    }
}

async fn api_v1_ipv4_assignment_space_create<T>(ext: Option<ExtensionExtractor<Server<T>>>, JsonExtractor(req): JsonExtractor<AssignmentSpaceIpv4>) -> Response<Body>
where
    T: DbConnection + Clone + Send + Sync + 'static,
{
    if let Some(ext) = ext {
        let store = ext.0.store();
        let res = match run_blocking_task(store.clone(), move |store| store.ipv4_assignments().create_space(&req)).await {
            Ok(space_id) => {
                if let Ok(space) = run_blocking_task(store.clone(), move |store| store.ipv4_assignments().get_space(space_id)).await {
                    let res = ApiResponse {
                        error: None,
                        result: Some(ApiResponseVariant::Ipv4AssignmentSpace(space)),
                    };
                    build_json_response(res, 200)
                } else {
                    let res = ApiResponse {
                        error: Some("Error creating assignment space".to_string()),
                        result: None,
                    };
                    build_json_response(res, 500)
                }
            },
            Err(e) => {
                let res = ApiResponse {
                    error: Some(format!("Error creating assignment space: {}", e)),
                    result: None,
                };
                build_json_response(res, 500)
            },
        };
        return res;
    } else {
        let res = ApiResponse {
            error: Some("Internal Server Error".to_string()),
            result: None,
        };
        return build_json_response(res, 500);
    }
}

async fn api_v1_ipv4_assignment_space_get<T>(ext: Option<ExtensionExtractor<Server<T>>>, PathExtractor(space_id): PathExtractor<i32>, user: Option<ExtensionExtractor<User>>) -> Response<Body>
where
    T: DbConnection + Clone + Send + Sync + 'static,
{
    if let Some(ext) = ext {
        let store = ext.0.store();
        let res = match run_blocking_task(store.clone(), move |store| store.ipv4_assignments().get_space(space_id)).await {
            Ok(space) => {
                if user.is_none() && space.space_visibility != ObjectVisibility::Public {
                    let res = ApiResponse {
                        error: Some("Assignment space not found".to_string()),
                        result: None,
                    };
                    return build_json_response(res, 404);
                }
                let res = ApiResponse {
                    error: None,
                    result: Some(ApiResponseVariant::Ipv4AssignmentSpace(space)),
                };
                build_json_response(res, 200)
            },
            Err(_) => {
                let res = ApiResponse {
                    error: Some("Assignment space not found".to_string()),
                    result: None,
                };
                build_json_response(res, 404)
            },
        };
        return res;
    } else {
        let res = ApiResponse {
            error: Some("Internal Server Error".to_string()),
            result: None,
        };
        return build_json_response(res, 500);
    }
}

async fn api_v1_ipv4_assignment_space_update<T>(ext: Option<ExtensionExtractor<Server<T>>>, PathExtractor(space_id): PathExtractor<i32>, JsonExtractor(req): JsonExtractor<MetadataUpdateRequest>) -> Response<Body>
where
    T: DbConnection + Clone + Send + Sync + 'static,
{
    if let Some(ext) = ext {
        let store = ext.0.store();
        let res = match run_blocking_task(store.clone(), move |store| store.ipv4_assignments().update_space(space_id, &req.name, &req.description)).await {
            Ok(_) => {
                if let Ok(space) = run_blocking_task(store.clone(), move |store| store.ipv4_assignments().get_space(space_id)).await {
                    let res = ApiResponse {
                        error: None,
                        result: Some(ApiResponseVariant::Ipv4AssignmentSpace(space)),
                    };
                    build_json_response(res, 200)
                } else {
                    let res = ApiResponse {
                        error: Some("Error updating assignment space".to_string()),
                        result: None,
                    };
                    build_json_response(res, 500)
                }
            },
            Err(e) => {
                let res = ApiResponse {
                    error: Some(format!("Error updating assignment space: {}", e)),
                    result: None,
                };
                build_json_response(res, 500)
            },
        };
        return res;
    } else {
        let res = ApiResponse {
            error: Some("Internal Server Error".to_string()),
            result: None,
        };
        return build_json_response(res, 500);
    }
}

async fn api_v1_ipv4_assignment_space_delete<T>(ext: Option<ExtensionExtractor<Server<T>>>, PathExtractor(space_id): PathExtractor<i32>) -> Response<Body>
where
    T: DbConnection + Clone + Send + Sync + 'static,
{
    if let Some(ext) = ext {
        let store = ext.0.store();
        let res = match run_blocking_task(store.clone(), move |store| store.ipv4_assignments().delete_space(space_id)).await {
            Ok(_) => {
                let res = ApiResponse {
                    error: None,
                    result: None,
                };
                build_json_response(res, 200)
            },
            Err(e) => {
                let res = ApiResponse {
                    error: Some(format!("Error deleting assignment space: {}", e)),
                    result: None,
                };
                build_json_response(res, 500)
            },
        };
        return res;
    } else {
        let res = ApiResponse {
            error: Some("Internal Server Error".to_string()),
            result: None,
        };
        return build_json_response(res, 500);
    }
}

async fn api_v1_ipv4_assignment_space_pool_list<T>(ext: Option<ExtensionExtractor<Server<T>>>, PathExtractor(space_id): PathExtractor<i32>, user: Option<ExtensionExtractor<User>>) -> Response<Body>
where
    T: DbConnection + Clone + Send + Sync + 'static,
{
    if let Some(ext) = ext {
        let store = ext.0.store();

        let space_id_copy = space_id;
        if let Ok(space) = run_blocking_task(store.clone(), move |store| store.ipv4_assignments().get_space(space_id_copy)).await {
            if user.is_none() && space.space_visibility != ObjectVisibility::Public {
                let res = ApiResponse {
                    error: Some("Assignment space not found".to_string()),
                    result: None,
                };
                return build_json_response(res, 404);
            }
        } else {
            let res = ApiResponse {
                error: Some("Assignment space not found".to_string()),
                result: None,
            };
            return build_json_response(res, 404);
        }

        let res = match run_blocking_task(store.clone(), move |store| store.ipv4_assignments().get_pools(space_id)).await {
            Ok(mut pools) => {
                if let None = user {
                    pools.retain(|pool| {
                        pool.pool_visibility == ObjectVisibility::Public
                    });
                }
                let res = ApiResponse {
                    error: None,
                    result: Some(ApiResponseVariant::Ipv4AssignmentPools(pools)),
                };
                build_json_response(res, 200)
            },
            Err(e) => {
                let res = ApiResponse {
                    error: Some(format!("Error listing pools: {}", e)),
                    result: None,
                };
                build_json_response(res, 400)
            },
        };
        return res;
    } else {
        let res = ApiResponse {
            error: Some("Internal Server Error".to_string()),
            result: None,
        };
        return build_json_response(res, 500);
    }
}

async fn api_v1_ipv4_assignment_space_pool_create<T>(ext: Option<ExtensionExtractor<Server<T>>>, PathExtractor(space_id): PathExtractor<i32>, JsonExtractor(req): JsonExtractor<AssignmentPoolIpv4>) -> Response<Body>
where
    T: DbConnection + Clone + Send + Sync + 'static,
{
    if req.assignment_space_id != space_id {
        let res = ApiResponse {
            error: Some("Assignment space ID mismatch".to_string()),
            result: None,
        };
        return build_json_response(res, 400);
    }
    if let Some(ext) = ext {
        let store = ext.0.store();
        let res = match run_blocking_task(store.clone(), move |store| store.ipv4_assignments().create_pool(&req)).await {
            Ok(pool_id) => {
                if let Ok(pool) = run_blocking_task(store.clone(), move |store| store.ipv4_assignments().get_pool(pool_id)).await {
                    let res = ApiResponse {
                        error: None,
                        result: Some(ApiResponseVariant::Ipv4AssignmentPool(pool)),
                    };
                    build_json_response(res, 200)
                } else {
                    let res = ApiResponse {
                        error: Some("Error creating pool".to_string()),
                        result: None,
                    };
                    build_json_response(res, 500)
                }
            },
            Err(e) => {
                let res = ApiResponse {
                    error: Some(format!("Error creating pool: {}", e)),
                    result: None,
                };
                build_json_response(res, 500)
            },
        };
        return res;
    } else {
        let res = ApiResponse {
            error: Some("Internal Server Error".to_string()),
            result: None,
        };
        return build_json_response(res, 500);
    }
}

#[allow(unused_variables)]
async fn api_v1_ipv4_assignment_space_pool_get<T>(ext: Option<ExtensionExtractor<Server<T>>>, PathExtractor((space_id, pool_id)): PathExtractor<(i32, i32)>, user: Option<ExtensionExtractor<User>>) -> Response<Body>
where
    T: DbConnection + Clone + Send + Sync + 'static,
{
    if let Some(ext) = ext {
        let store = ext.0.store();

        let res = match run_blocking_task(store.clone(), move |store| store.ipv4_assignments().get_pool(pool_id)).await {
            Ok(pool) => {
                let space_id = pool.assignment_space_id;
                if let Ok(space) = run_blocking_task(store.clone(), move |store| store.ipv4_assignments().get_space(space_id)).await {
                    if user.is_none() && space.space_visibility != ObjectVisibility::Public {
                        let res = ApiResponse {
                            error: Some("Pool not found".to_string()),
                            result: None,
                        };
                        return build_json_response(res, 404);
                    }
                } else {
                    let res = ApiResponse {
                        error: Some("Pool not found".to_string()),
                        result: None,
                    };
                    return build_json_response(res, 404);
                }

                if user.is_none() && pool.pool_visibility != ObjectVisibility::Public {
                    let res = ApiResponse {
                        error: Some("Pool not found".to_string()),
                        result: None,
                    };
                    return build_json_response(res, 404);
                }
                let res = ApiResponse {
                    error: None,
                    result: Some(ApiResponseVariant::Ipv4AssignmentPool(pool)),
                };
                build_json_response(res, 200)
            },
            Err(_) => {
                let res = ApiResponse {
                    error: Some("Pool not found".to_string()),
                    result: None,
                };
                build_json_response(res, 404)
            },
        };
        return res;
    } else {
        let res = ApiResponse {
            error: Some("Internal Server Error".to_string()),
            result: None,
        };
        return build_json_response(res, 500);
    }
}

#[allow(unused_variables)]
async fn api_v1_ipv4_assignment_space_pool_update<T>(ext: Option<ExtensionExtractor<Server<T>>>, PathExtractor((space_id, pool_id)): PathExtractor<(i32, i32)>, JsonExtractor(req): JsonExtractor<MetadataUpdateRequest>) -> Response<Body>
where
    T: DbConnection + Clone + Send + Sync + 'static,
{
    if let Some(ext) = ext {
        let store = ext.0.store();
        let res = match run_blocking_task(store.clone(), move |store| store.ipv4_assignments().update_pool(pool_id, &req.name, &req.description)).await {
            Ok(_) => {
                if let Ok(pool) = run_blocking_task(store.clone(), move |store| store.ipv4_assignments().get_pool(pool_id)).await {
                    let res = ApiResponse {
                        error: None,
                        result: Some(ApiResponseVariant::Ipv4AssignmentPool(pool)),
                    };
                    build_json_response(res, 200)
                } else {
                    let res = ApiResponse {
                        error: Some("Error updating pool".to_string()),
                        result: None,
                    };
                    build_json_response(res, 500)
                }
            },
            Err(e) => {
                let res = ApiResponse {
                    error: Some(format!("Error updating pool: {}", e)),
                    result: None,
                };
                build_json_response(res, 500)
            },
        };
        return res;
    } else {
        let res = ApiResponse {
            error: Some("Internal Server Error".to_string()),
            result: None,
        };
        return build_json_response(res, 500);
    }
}

#[allow(unused_variables)]
async fn api_v1_ipv4_assignment_space_pool_delete<T>(ext: Option<ExtensionExtractor<Server<T>>>, PathExtractor((space_id, pool_id)): PathExtractor<(i32, i32)>) -> Response<Body>
where
    T: DbConnection + Clone + Send + Sync + 'static,
{
    if let Some(ext) = ext {
        let store = ext.0.store();
        let res = match run_blocking_task(store.clone(), move |store| store.ipv4_assignments().delete_pool(pool_id)).await {
            Ok(_) => {
                let res = ApiResponse {
                    error: None,
                    result: None,
                };
                build_json_response(res, 200)
            },
            Err(e) => {
                let res = ApiResponse {
                    error: Some(format!("Error deleting pool: {}", e)),
                    result: None,
                };
                build_json_response(res, 500)
            },
        };
        return res;
    } else {
        let res = ApiResponse {
            error: Some("Internal Server Error".to_string()),
            result: None,
        };
        return build_json_response(res, 500);
    }
}

#[allow(unused_variables)]
async fn api_v1_ipv4_assignment_space_pool_assignment_list<T>(ext: Option<ExtensionExtractor<Server<T>>>, PathExtractor((space_id, pool_id)): PathExtractor<(i32, i32)>, user: Option<ExtensionExtractor<User>>) -> Response<Body>
where
    T: DbConnection + Clone + Send + Sync + 'static,
{
    if let Some(ext) = ext {
        let store = ext.0.store();

        let pool_id_copy = pool_id;
        if let Ok(pool) = run_blocking_task(store.clone(), move |store| store.ipv4_assignments().get_pool(pool_id_copy)).await {
            if user.is_none() && pool.pool_visibility != ObjectVisibility::Public {
                let res = ApiResponse {
                    error: Some("Pool not found".to_string()),
                    result: None,
                };
                return build_json_response(res, 404);
            }

            let space_id = pool.assignment_space_id;

            if let Ok(space) = run_blocking_task(store.clone(), move |store| store.ipv4_assignments().get_space(space_id)).await {
                if space.id != pool.assignment_space_id {
                    let res = ApiResponse {
                        error: Some("Pool not found".to_string()),
                        result: None,
                    };
                    return build_json_response(res, 404);
                }
            } else {
                let res = ApiResponse {
                    error: Some("Pool not found".to_string()),
                    result: None,
                };
                return build_json_response(res, 404);
            }
        } else {
            let res = ApiResponse {
                error: Some("Pool not found".to_string()),
                result: None,
            };
            return build_json_response(res, 404);
        }

        let res = match run_blocking_task(store.clone(), move |store| store.ipv4_assignments().get_assignments(pool_id)).await {
            Ok(mut assignments) => {
                if let None = user {
                    assignments.retain(|assignment| {
                        assignment.assignment_visibility == ObjectVisibility::Public
                    });
                }
                let res = ApiResponse {
                    error: None,
                    result: Some(ApiResponseVariant::Ipv4Assignments(assignments)),
                };
                build_json_response(res, 200)
            },
            Err(e) => {
                let res = ApiResponse {
                    error: Some(format!("Error listing assignments: {}", e)),
                    result: None,
                };
                build_json_response(res, 400)
            },
        };
        return res;
    } else {
        let res = ApiResponse {
            error: Some("Internal Server Error".to_string()),
            result: None,
        };
        return build_json_response(res, 500);
    }
}

#[allow(unused_variables)]
async fn api_v1_ipv4_assignment_space_pool_assignment_create<T>(ext: Option<ExtensionExtractor<Server<T>>>, PathExtractor((space_id, pool_id)): PathExtractor<(i32, i32)>, JsonExtractor(req): JsonExtractor<AssignmentIpv4>) -> Response<Body>
where
    T: DbConnection + Clone + Send + Sync + 'static,
{
    if req.assignment_pool_id != pool_id {
        let res = ApiResponse {
            error: Some("Assignment pool ID mismatch".to_string()),
            result: None,
        };
        return build_json_response(res, 400);
    }
    if let Some(ext) = ext {
        let store = ext.0.store();
        let res = match run_blocking_task(store.clone(), move |store| store.ipv4_assignments().create_assignment(&req)).await {
            Ok(assignment_id) => {
                if let Ok(assignment) = run_blocking_task(store.clone(), move |store| store.ipv4_assignments().get_assignment(assignment_id)).await {
                    let res = ApiResponse {
                        error: None,
                        result: Some(ApiResponseVariant::Ipv4Assignment(assignment)),
                    };
                    build_json_response(res, 200)
                } else {
                    let res = ApiResponse {
                        error: Some("Error creating assignment".to_string()),
                        result: None,
                    };
                    build_json_response(res, 500)
                }
            },
            Err(e) => {
                let res = ApiResponse {
                    error: Some(format!("Error creating assignment: {}", e)),
                    result: None,
                };
                build_json_response(res, 500)
            },
        };
        return res;
    } else {
        let res = ApiResponse {
            error: Some("Internal Server Error".to_string()),
            result: None,
        };
        return build_json_response(res, 500);
    }
}

#[allow(unused_variables)]
async fn api_v1_ipv4_assignment_space_pool_assignment_get<T>(ext: Option<ExtensionExtractor<Server<T>>>, PathExtractor((space_id, pool_id, assignment_id)): PathExtractor<(i32, i32, i32)>, user: Option<ExtensionExtractor<User>>) -> Response<Body>
where
    T: DbConnection + Clone + Send + Sync + 'static,
{
    if let Some(ext) = ext {
        let store = ext.0.store();
        let res = match run_blocking_task(store.clone(), move |store| store.ipv4_assignments().get_assignment(assignment_id)).await {
            Ok(assignment) => {
                let pool_id = assignment.assignment_pool_id;
                if let Ok(pool) = run_blocking_task(store.clone(), move |store| store.ipv4_assignments().get_pool(pool_id)).await {
                    if user.is_none() && pool.pool_visibility != ObjectVisibility::Public {
                        let res = ApiResponse {
                            error: Some("Assignment not found".to_string()),
                            result: None,
                        };
                        return build_json_response(res, 404);
                    }

                    let space_id = pool.assignment_space_id;

                    if let Ok(space) = run_blocking_task(store.clone(), move |store| store.ipv4_assignments().get_space(space_id)).await {
                        if space.id != pool.assignment_space_id {
                            let res = ApiResponse {
                                error: Some("Assignment not found".to_string()),
                                result: None,
                            };
                            return build_json_response(res, 404);
                        }
                    } else {
                        let res = ApiResponse {
                            error: Some("Assignment not found".to_string()),
                            result: None,
                        };
                        return build_json_response(res, 404);
                    }
                } else {
                    let res = ApiResponse {
                        error: Some("Assignment not found".to_string()),
                        result: None,
                    };
                    return build_json_response(res, 404);
                }

                if user.is_none() && assignment.assignment_visibility != ObjectVisibility::Public {
                    let res = ApiResponse {
                        error: Some("Assignment not found".to_string()),
                        result: None,
                    };
                    return build_json_response(res, 404);
                }
                let res = ApiResponse {
                    error: None,
                    result: Some(ApiResponseVariant::Ipv4Assignment(assignment)),
                };
                build_json_response(res, 200)
            },
            Err(_) => {
                let res = ApiResponse {
                    error: Some("Assignment not found".to_string()),
                    result: None,
                };
                build_json_response(res, 404)
            },
        };
        return res;
    } else {
        let res = ApiResponse {
            error: Some("Internal Server Error".to_string()),
            result: None,
        };
        return build_json_response(res, 500);
    }
}

#[allow(unused_variables)]
async fn api_v1_ipv4_assignment_space_pool_assignment_update<T>(ext: Option<ExtensionExtractor<Server<T>>>, PathExtractor((space_id, pool_id, assignment_id)): PathExtractor<(i32, i32, i32)>, JsonExtractor(req): JsonExtractor<MetadataUpdateRequest>) -> Response<Body>
where
    T: DbConnection + Clone + Send + Sync + 'static,
{
    if let Some(ext) = ext {
        let store = ext.0.store();
        let res = match run_blocking_task(store.clone(), move |store| store.ipv4_assignments().update_assignment(assignment_id, &req.name, &req.description)).await {
            Ok(_) => {
                if let Ok(assignment) = run_blocking_task(store.clone(), move |store| store.ipv4_assignments().get_assignment(assignment_id)).await {
                    let res = ApiResponse {
                        error: None,
                        result: Some(ApiResponseVariant::Ipv4Assignment(assignment)),
                    };
                    build_json_response(res, 200)
                } else {
                    let res = ApiResponse {
                        error: Some("Error updating assignment".to_string()),
                        result: None,
                    };
                    build_json_response(res, 500)
                }
            },
            Err(e) => {
                let res = ApiResponse {
                    error: Some(format!("Error updating assignment: {}", e)),
                    result: None,
                };
                build_json_response(res, 500)
            },
        };
        return res;
    } else {
        let res = ApiResponse {
            error: Some("Internal Server Error".to_string()),
            result: None,
        };
        return build_json_response(res, 500);
    }
}

#[allow(unused_variables)]
async fn api_v1_ipv4_assignment_space_pool_assignment_delete<T>(ext: Option<ExtensionExtractor<Server<T>>>, PathExtractor((space_id, pool_id, assignment_id)): PathExtractor<(i32, i32, i32)>) -> Response<Body>
where
    T: DbConnection + Clone + Send + Sync + 'static,
{
    if let Some(ext) = ext {
        let store = ext.0.store();
        let res = match run_blocking_task(store.clone(), move |store| store.ipv4_assignments().delete_assignment(assignment_id)).await {
            Ok(_) => {
                let res = ApiResponse {
                    error: None,
                    result: None,
                };
                build_json_response(res, 200)
            },
            Err(e) => {
                let res = ApiResponse {
                    error: Some(format!("Error deleting assignment: {}", e)),
                    result: None,
                };
                build_json_response(res, 500)
            },
        };
        return res;
    } else {
        let res = ApiResponse {
            error: Some("Internal Server Error".to_string()),
            result: None,
        };
        return build_json_response(res, 500);
    }
}

pub fn build_router<T>() -> Router<Server<T>>
where 
    T: DbConnection + Clone + Send + Sync + 'static,
{
    let mut router = Router::new();

    router = router.route("/assignment_space", get(api_v1_ipv4_assignment_space_list::<T>).layer(AuthHandler::<T>::new_layer()));
    router = router.route("/assignment_space", post(api_v1_ipv4_assignment_space_create::<T>).layer(AuthHandler::<T>::new_auth_required_layer()));
    router = router.route("/assignment_space/:space_id", get(api_v1_ipv4_assignment_space_get::<T>).layer(AuthHandler::<T>::new_layer()));
    router = router.route("/assignment_space/:space_id", put(api_v1_ipv4_assignment_space_update::<T>).layer(AuthHandler::<T>::new_auth_required_layer()));
    router = router.route("/assignment_space/:space_id", delete(api_v1_ipv4_assignment_space_delete::<T>).layer(AuthHandler::<T>::new_auth_required_layer()));

    router = router.route("/assignment_space/:space_id/pool", get(api_v1_ipv4_assignment_space_pool_list::<T>).layer(AuthHandler::<T>::new_layer()));
    router = router.route("/assignment_space/:space_id/pool", post(api_v1_ipv4_assignment_space_pool_create::<T>).layer(AuthHandler::<T>::new_auth_required_layer()));
    router = router.route("/assignment_space/:space_id/pool/:pool_id", get(api_v1_ipv4_assignment_space_pool_get::<T>).layer(AuthHandler::<T>::new_layer()));
    router = router.route("/assignment_space/:space_id/pool/:pool_id", put(api_v1_ipv4_assignment_space_pool_update::<T>).layer(AuthHandler::<T>::new_auth_required_layer()));
    router = router.route("/assignment_space/:space_id/pool/:pool_id", delete(api_v1_ipv4_assignment_space_pool_delete::<T>).layer(AuthHandler::<T>::new_auth_required_layer()));

    router = router.route("/assignment_space/:space_id/pool/:pool_id/assignment", get(api_v1_ipv4_assignment_space_pool_assignment_list::<T>).layer(AuthHandler::<T>::new_layer()));
    router = router.route("/assignment_space/:space_id/pool/:pool_id/assignment", post(api_v1_ipv4_assignment_space_pool_assignment_create::<T>).layer(AuthHandler::<T>::new_auth_required_layer()));
    router = router.route("/assignment_space/:space_id/pool/:pool_id/assignment/:assignment_id", get(api_v1_ipv4_assignment_space_pool_assignment_get::<T>).layer(AuthHandler::<T>::new_layer()));
    router = router.route("/assignment_space/:space_id/pool/:pool_id/assignment/:assignment_id", put(api_v1_ipv4_assignment_space_pool_assignment_update::<T>).layer(AuthHandler::<T>::new_auth_required_layer()));
    router = router.route("/assignment_space/:space_id/pool/:pool_id/assignment/:assignment_id", delete(api_v1_ipv4_assignment_space_pool_assignment_delete::<T>).layer(AuthHandler::<T>::new_auth_required_layer()));

    router = router.fallback(fallback_handler());
    router
}
