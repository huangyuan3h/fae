#[cfg(test)]
mod chat_functionality_tests {
    use axum::extract::State;
    use tokio_stream::StreamExt;
    use serde_json::json;
    use std::sync::Arc;
    use sqlx::{Pool, SqlitePool};
    use futures::Stream;

    use crate::services::{AgentChatRequest, agent_chat_handler, agent_stream_chat_handler};
    use crate::AppState;

    // Test for the regular chat handler
    #[tokio::test]
    async fn test_agent_chat_handler_basic_functionality() {
        // Create mock app state (not connected to the actual functionality for this test)
        let dummy_pool = create_dummy_db_pool().await;
let app_state = AppState {
            db_pool: Arc::new(dummy_pool),
            llm_log_dir: "./logs/llm".to_string(),
        };

        let payload = json!({
            "agentId": "test-id-123",
            "message": "Hello, how are you?"
        });
        
        let agent_chat_req: AgentChatRequest = serde_json::from_value(payload).unwrap();
        
        // Call the handler function
        let result = agent_chat_handler(
            State(app_state),
            axum::Json(agent_chat_req)
        ).await.unwrap();

        // Assertions
        assert!(result.message.contains("test-id-123"));
        assert!(result.message.contains("Hello, how are you?"));
        assert_eq!(result.model, "mock-model");
        assert!(!result.message.is_empty());
    }

    #[tokio::test]
    async fn test_agent_stream_chat_handler_produces_events() {
        // Create mock app state
        let dummy_pool = create_dummy_db_pool().await;
let app_state = AppState {
            db_pool: Arc::new(dummy_pool),
            llm_log_dir: "./logs/llm".to_string(),
        };

        let payload = json!({
            "agentId": "stream-agent-test",
            "message": "Process this message"
        });
        
        let agent_chat_req: AgentChatRequest = serde_json::from_value(payload).unwrap();
        
        // Create a stream from the handler
        let response_stream = agent_stream_chat_handler(
            State(app_state),
            axum::Json(agent_chat_req)
        );

        // Convert to raw stream and test first few events
        let mut stream = response_stream.into_stream();
        
        // Collect the first few events to inspect
        let mut events_collected = Vec::new();
        for _ in 0..10 { // Test first 10 events
            if let Some(event_res) = stream.next().await {
                match event_res {
                    Ok(event) => {
                        if let Some(event_data) = event.data() {
                            events_collected.push(event_data.clone());
                        }
                    },
                    Err(_) => break, // Stop if error occurs in stream
                }
            } else {
                break; // Stream ended
            }
        }

        // Assertions
        assert!(!events_collected.is_empty(), "Should collect at least one event");
        
        // Check that we have different kinds of events (think, tool, chunk)
        let has_think_events = events_collected.iter().any(|e| e.contains(r#""type":"think""#));
        let has_tool_events = events_collected.iter().any(|e| e.contains("toolCallId"));
        let has_chunk_events = events_collected.iter().any(|e| e.contains(r#""type":"chunk""#));
        
        assert!(has_think_events, "Should have think events");
        assert!(has_tool_events, "Should have tool events");  
        assert!(has_chunk_events, "Should have chunk events");
        
        // All events should contain valid JSON structure with type field
        for event_data in &events_collected {
            assert!(event_data.contains("type"), "Each event should have a type field");
        }
    }
    
    #[tokio::test]
    async fn test_stream_handles_different_messages() {
        let dummy_pool = create_dummy_db_pool().await;
let app_state = AppState {
            db_pool: Arc::new(dummy_pool),
            llm_log_dir: "./logs/llm".to_string(),
        };

        let test_cases = vec![
            "Simple test message",
            "What is the meaning of life?", 
            "Analyze and report search results",
            "Perform complex calculation"
        ];

        for (i, message) in test_cases.iter().enumerate() {
            let payload = json!({
                "agentId": format!("test-agent-{}", i),
                "message": message
            });
            
            let agent_chat_req: AgentChatRequest = serde_json::from_value(payload).unwrap();
            
            let response_stream = agent_stream_chat_handler(
                State(app_state.clone()),
                axum::Json(agent_chat_req)
            );

            let mut stream = response_stream.into_stream();
            let mut events = Vec::new();
            
            // Collect up to 3 events for each test
            for _ in 0..3 {
                if let Some(event_res) = stream.next().await {
                    match event_res {
                        Ok(event) => {
                            if let Some(event_data) = event.data() {
                                events.push(event_data.clone());
                            }
                        },
                        Err(_) => break,
                    }
                } else {
                    break;
                }
            }
            
            // Each message should produce a sequence of events
            assert!(!events.is_empty(), "Message '{}' should generate events", message);
            
            // All test cases should follow the same pattern (think -> possibly tools -> chunks)
            let has_think = events.iter().any(|e| e.contains(r#""type":"think""#));
            assert!(has_think, "Test case '{}' should have think events", message);
        }
    }

    // Helper function to create a dummy db pool for testing
    async fn create_dummy_db_pool() -> SqlitePool {
        let conn = SqlitePool::connect("sqlite::memory:").await.unwrap();
        
        // Optionally run migrations if testing that functionality too
        sqlx::migrate!("./migrations")
            .run(&conn)
            .await
            .expect("Migrations to apply");
        
        conn
    }
}

// Make sure our state is cloneable for tests
impl Clone for AppState {
    fn clone(&self) -> Self {
        AppState {
            db_pool: self.db_pool.clone()
        }
    }
}