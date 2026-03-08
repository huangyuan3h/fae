// Handler for streaming chat responses to match /api/chat/stream expectation - returns SSE
pub async fn agent_stream_chat_handler(
    State(state): State<crate::AppState>,
    AxumJson(payload): AxumJson<AgentChatRequest>,
) -> Sse<impl Stream<Item = Result<Event, std::convert::Infallible>>> {
    let agent_id = payload.agent_id;
    let message = payload.message;
    
    // Fetch agent details from database to determine actual skills
    let agent_result = sqlx::query_as!(
        crate::models::Agent,
        r#"
        SELECT 
            id as "id!",
            name as "name!",
            provider as "provider!",
            provider_config_id as "provider_config_id?",
            model as "model!",
            system_prompt as "system_prompt?",
            avatar_url as "avatar_url?",
            skills as "skills!: Vec<String>",
            created_at as "created_at!",
            updated_at as "updated_at!"
        FROM agents 
        WHERE id = ?
        "#,
        agent_id
    )
    .fetch_one(&state.db_pool)
    .await;

    let stream = async_stream::stream! {
        // Send thinking stages to simulate AI processing
        let thinking_steps = vec![
            "Processing user request. ",
            "Analyzing the query for relevant actions. ", 
            "Considering which tools would be most helpful. ",
            "Preparing a structured response. "
        ];
        
        for step in thinking_steps {
            tokio::time::sleep(Duration::from_millis(200)).await;
            yield Ok(Event::default()
                .event("data")
                .data(format!(r#"{{"type":"think","content":"{}"}}"#, step)));
        }

        // Determine if agent has skills, then simulate tools matching real skills
        match agent_result {
            Ok(agent) => {
                let agent_skills = agent.skills;
                
                if !agent_skills.is_empty() {
                    for skill_id in &agent_skills {
                        // For each skill the agent has, simulate calling it

                        // Generate a unique tool call ID
                        let tool_call_id = format!("call_{}", &uuid::Uuid::new_v4().to_string()[..8]);
                        
                        yield Ok(Event::default()
                            .event("data")
                            .data(format!(
                                r#"{{"type":"tool-input-start","toolCallId":"{}","toolName":"{}"}}"#, 
                                tool_call_id, skill_id
                            )));
                        
                        tokio::time::sleep(Duration::from_millis(300)).await;
                        
                        let esc_message = message.replace("\"", "'");
                        yield Ok(Event::default()
                            .event("data")
                            .data(format!(
                                r#"{{"type":"tool-call","toolCallId":"{}","toolName":"{}","input":{{"query":"{}","taskId":"analyze"}}}}"#, 
                                tool_call_id, skill_id, esc_message
                            )));
                        
                        tokio::time::sleep(Duration::from_millis(500)).await;
                        
                        yield Ok(Event::default()
                            .event("data")
                            .data(format!(
                                r#"{{"type":"tool-result","toolCallId":"{}","toolName":"{}","output":{{"status":"success","data":"Results for {} operation on {}", "query":"{}", "metadata":{{"execTime":123,"source":"skill_impl"}}}}}}"#, 
                                tool_call_id, skill_id, skill_id, esc_message, esc_message
                            )));
                    }
                } else {
                    // If agent has no skills, send a simple response event
                    yield Ok(Event::default()
                        .event("data")
                        .data(r#"{"type":"chunk","content":"The agent processed your request using general knowledge. "}"#));
                }
            },
            Err(_e) => {
                // If agent doesn't exist, send a notification
                yield Ok(Event::default()
                    .event("data")
                    .data(r#"{"type":"chunk","content":"Processing with default capabilities. Agent may not exist. "}"#));
            }
        }

        // Generate response segments based on the input message
        let response_segments = vec![
            format!("I have processed your request about ("),
            message.clone(),
            format!("). "),
            format!("Using available skills, ") ,
            format!("I have analyzed and synthesized the required information. "),
            "I hope this addresses your query. Is there anything else I can assist with?".to_string()
        ];
        
        for segment in response_segments {
            tokio::time::sleep(Duration::from_millis(100)).await;
            yield Ok(Event::default()
                .event("data")
                .data(format!(r#"{{"type":"chunk","content":"{}"}}"#, segment))); 
        }
    };
    
    Sse::new(stream).keep_alive(KeepAlive::default())
}
