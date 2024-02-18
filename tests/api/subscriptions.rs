use crate::helpers::spawn_app;

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app = spawn_app().await;

    // Check that the response is the expected one
    let body = String::from("name=le%20guin&email=ursula_le_guin%40gmail.com");
    let response = app.post_subscriptions(body).await;
    assert_eq!(200, response.status().as_u16());

    // Check that the result is actually saved in the database
    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let app = spawn_app().await;
    let test_cases = vec![
        (String::from("name=le%20guin"), "missing the email"),
        (
            String::from("email=ursula_le_guin%40gmail.com"),
            "missing the name",
        ),
        (String::from(""), "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = app.post_subscriptions(invalid_body).await;
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}",
            error_message
        );
    }
}

#[tokio::test]
async fn subscribe_returns_a_400_when_fields_are_present_but_invalid() {
    let app = spawn_app().await;
    let test_cases = vec![
        (String::from("name=Ursula&email="), "empty email"),
        (
            String::from("name=&email=ursula_le_guin%40gmail.com"),
            "empty name",
        ),
        (
            String::from("name=Ursula&email=definitely-not-an-email"),
            "invalid email",
        ),
    ];

    for (body, description) in test_cases {
        // Act
        let response = app.post_subscriptions(body).await;
        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return a 400 Bad Request when the payload was {}.",
            description
        );
    }
}
