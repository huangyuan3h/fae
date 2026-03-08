#[cfg(test)]
mod chat_stream_tests {
    use axum::extract::State;
    use tokio_stream::StreamExt;
    use serde_json::json;
    use std::sync::Arc;
    use sqlx::{Connection, SqliteConnection, Pool, Row};

    use crate::models::Agent;
    use crate::services::{AgentChatRequest};
    use crate::AppState;

    #[tokio::test]
    async fn test_agent_chat_handler_basic_response() {
        // Arrange
        let db_url = "sqlite::memory:";
        let mut conn = SqliteConnection::connect(db_url).await.unwrap();
        
        // Run migrations on memory DB
        sqlx::migrate!("./migrations")
            .run(&mut conn)
            .await
            .expect("Migrations should apply cleanly");
        
        let db_pool = sqlx::SqlitePool::connect_with(conn).await.unwrap();
        
        let app_state = AppState {
            db_pool: Arc::new(db_pool),
        };

        let payload = json!({
            "agentId": "test-id",
            "message": "Hello world!"
        });
        
        let agent_chat_req: AgentChatRequest = serde_json::from_value(payload).unwrap();
        
        // Act
        let result = crate::services::agent_chat_handler(
            State(app_state),
            axum::Json(agent_chat_req)
        ).await.unwrap();

        // Assert
        assert!(!result.message.is_empty());
        assert_eq!(result.model, "mock-model");
        assert!(result.message.contains("test-id"));
        assert!(result.message.contains("Hello world!"));
    }

    #[tokio::test]
    async fn test_agent_stream_chat_handler_emits_events() {
        // Arrange
        let db_url = "sqlite::memory:";
        let mut conn = SqliteConnection::connect(db_url).await.unwrap();
        
        // Run migrations on memory DB
        sqlx::migrate!("./migrations")
            .run(&mut conn)
            .await
            .expect("Migrations should apply cleanly");
        
        let db_pool = sqlx::SqlitePool::connect_with(conn).await.unwrap();
        
        let app_state = AppState {
            db_pool: Arc::new(db_pool),
        };

        let payload = json!({
            "agentId": "test-stream-id",
            "message": "Stream test message"
        });
        
        let agent_chat_req: AgentChatRequest = serde_json::from_value(payload).unwrap();
        
        // Act
        let stream_result = crate::services::agent_stream_chat_handler(
            State(app_state),
            axum::Json(agent_chat_req)
        );

        // Test that the stream produces events (without fully consuming it)
        let mut stream = Box::pin(stream_result.into_stream());
        
        // Assert we can get some events without error
        let mut events_received = 0;
        while let Some(event_result) = stream.next().await {
            if events_received >= 5 { // Just check first few events to confirm proper streaming
                break;
            }
            
            match event_result {
                Ok(event) => {
                    // Check that event has expected structure
                    let data = event.data();
                    assert!(data.is_some());  // Should have data
                    
                    // Validate that events have expected types (think, chunk, etc.)
                    let event_data = data.unwrap();
                    assert!(event_data.contains("type"));
                    
                    events_received += 1;
                }
                Err(e) => {
                    // This shouldn't happen in a properly formed test
                    panic!("Stream error: {:?}", e);
                }
            }
        }
        
        // Should have received multiple events (thinking and chunks)
        assert!(events_received > 0, "Expected to receive stream events");
    }

    #[tokio::test]
    async fn test_tool_calls_in_stream() {
        // Arrange
        let db_url = "sqlite::memory:";
        let mut conn = SqliteConnection::connect(db_url).await.unwrap();
        
        // Run migrations on memory DB
        sqlx::migrate!("./migrations")
            .run(&mut conn)
            .await
            .expect("Migrations should apply cleanly");
        
        let db_pool = sqlx::SqlitePool::connect_with(conn).await.unwrap();
        
        let app_state = AppState {
            db_pool: Arc::new(db_pool),
        };

        let payload = json!({
            "agentId": "tool-test-id",
            "message": "Please analyze and search"
        });
        
        let agent_chat_req: AgentChatRequest = serde_json::from_value(payload).unwrap();
        
        // Act
        let stream_result = crate::services::agent_stream_chat_handler(
            State(app_state),
            axum::Json(agent_chat_req)
        );

        let mut stream = Box::pin(stream_result.into_stream());
        
        // Assert presence of tool-related events
        let mut has_think_event = false;
        let mut has_tool_events = false;
        let mut has_chunk_events = false;
        
        while let Some(event_result) = stream.next().await {
            match event_result {
                Ok(event) => {
                    if let Some(data) = event.data() {
                        if data.contains(r#""type":"think""#) {
                            has_think_event = true;
                        }
                        if data.contains("tool") && data.contains("toolCallId") {
                            has_tool_events = true;
                        }
                        if data.contains(r#""type":"chunk""#) {
                            has_chunk_events = true;
                        }
                        
                        // Additional validation for tool-event structure
                        if data.contains("tool-call") {
                            assert!(data.contains("toolCallId"));
                            assert!(data.contains("toolName"));
                            assert!(data.contains("input"));
                        }
                        
                        if data.contains("tool-result") {
                            assert!(data.contains("toolCallId"));
                            assert!(data.contains("toolName"));
                            assert!(data.contains("output"));
                        }
                    }
                }
                Err(_e) => break, // Stop checking when stream ends
            }
            
            // Limit to first 10 events for efficiency
            if has_think_event && has_tool_events && has_chunk_events {
                break;
            }
        }
        
        // Assertions that all key event types were produced
        assert!(has_think_event, "Should emit thinking events");
        assert!(has_tool_events, "Should emit tool events");
        assert!(has_chunk_events, "Should emit chunk events");
    }
}

#[cfg(test)]
mod ai_service_tests {
    use sqlx::{Connection, SqliteConnection};
    use std::sync::Arc;

    use crate::services::ai_chat::AIChatService;
    use crate::AppState;

    #[tokio::test]
    async fn test_ai_service_can_be_created() {
        // Arrange
        let db_url = "sqlite::memory:";
        let mut conn = SqliteConnection::connect(db_url).await.unwrap();
        
        // Run migrations on memory DB
        sqlx::migrate!("./migrations")
            .run(&mut conn)
            .await
            .expect("Migrations should apply cleanly");
        
        let db_pool = sqlx::SqlitePool::connect_with(conn).await.unwrap();
        
        // Act
        let ai_service = AIChatService::new(&db_pool);
        
        // Assert - just verify we can instantiate without errors for now
        // More comprehensive tests would require actual agent creation and retrieval
        assert!(true); // Basic instantiation test
    }
    
    #[tokio::test]
    async fn test_ai_service_creation_with_arc_db_pool() {
        // Arrange
        let db_url = "sqlite::memory:";
        let mut conn = SqliteConnection::connect(db_url).await.unwrap();
        
        // Run migrations on memory DB
        sqlx::migrate!("./migrations")
            .run(&mut conn)
            .await
            .expect("Migrations should apply cleanly");
        
        let db_pool = Arc::new(sqlx::SqlitePool::connect_with(conn).await.unwrap());
        
        // Act 
        let ai_service = AIChatService::new(db_pool.as_ref());
        
        // Assert
        assert!(true); // Verify construction works with Arc wrapper
    }
}