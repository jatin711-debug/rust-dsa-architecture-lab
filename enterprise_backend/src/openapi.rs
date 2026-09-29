use crate::domain::{
    order::{CreateOrderDto, OrderItemInputDto, OrderItemResponse, OrderResponse},
    product::{CreateProductDto, Product, UpdateProductDto},
    user::{AuthTokenResponse, CreateUserDto, LoginUserDto, Role, UserResponse},
};
use utoipa::{
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify, OpenApi,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::health_routes::health_check,
        crate::routes::auth_routes::register,
        crate::routes::auth_routes::login,
        crate::routes::auth_routes::me,
        crate::routes::product_routes::list_products,
        crate::routes::product_routes::get_product,
        crate::routes::product_routes::create_product,
        crate::routes::product_routes::update_product,
        crate::routes::order_routes::create_order,
        crate::routes::order_routes::list_user_orders,
        crate::routes::order_routes::get_order,
    ),
    components(
        schemas(
            CreateUserDto,
            LoginUserDto,
            UserResponse,
            AuthTokenResponse,
            Role,
            Product,
            CreateProductDto,
            UpdateProductDto,
            CreateOrderDto,
            OrderItemInputDto,
            OrderResponse,
            OrderItemResponse
        )
    ),
    tags(
        (name = "Health", description = "System diagnostic endpoints"),
        (name = "Auth", description = "Authentication & User registration"),
        (name = "Products", description = "Catalog & Inventory management"),
        (name = "Orders", description = "Atomic order checkout & history")
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}
