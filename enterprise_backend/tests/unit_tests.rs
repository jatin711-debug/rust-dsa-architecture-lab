use enterprise_backend::services::auth_service::AuthService;
use uuid::Uuid;

#[test]
fn registration_rejects_client_supplied_role() {
    use enterprise_backend::domain::user::CreateUserDto;
    let payload = serde_json::json!({
        "email": "user@example.com",
        "password": "example-password",
        "full_name": "Example User",
        "role": "StoreAdmin"
    });
    assert!(serde_json::from_value::<CreateUserDto>(payload).is_err());
}

#[test]
fn test_password_hashing_and_verification() {
    let password = "SuperSecretPassword123!";
    let hash = AuthService::hash_password(password).expect("Hashing failed");

    assert!(hash.starts_with("$argon2id$"));

    let is_valid = AuthService::verify_password(password, &hash).expect("Verification failed");
    assert!(
        is_valid,
        "Password verification should succeed for correct password"
    );

    let is_invalid =
        AuthService::verify_password("WrongPassword!", &hash).expect("Verification failed");
    assert!(
        !is_invalid,
        "Password verification should fail for wrong password"
    );
}

#[test]
fn test_jwt_token_generation_and_validation() {
    let user_id = Uuid::new_v4();
    let email = "test_user@example.com";
    let role = "StoreAdmin";
    let secret = "my_jwt_secret_key_for_testing_12345";
    let exp_hours = 24;

    let token = AuthService::generate_jwt(user_id, email, role, secret, exp_hours)
        .expect("JWT generation failed");

    assert!(!token.is_empty());

    let claims = AuthService::verify_jwt(&token, secret).expect("JWT verification failed");
    assert_eq!(claims.sub, user_id);
    assert_eq!(claims.email, email);
    assert_eq!(claims.role, role);
}
