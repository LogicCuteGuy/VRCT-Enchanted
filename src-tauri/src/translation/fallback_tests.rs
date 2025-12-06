// Tests for translation fallback logic
// Requirements: 4.4

#[cfg(test)]
mod tests {
    use super::super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    // Mock translator that always fails
    struct FailingTranslator {
        name: String,
    }

    #[async_trait::async_trait]
    impl Translator for FailingTranslator {
        async fn translate(
            &self,
            _text: &str,
            _source_lang: &str,
            _target_lang: &str,
        ) -> Result<String> {
            Err(VrctError::Translation(format!(
                "{} translation failed",
                self.name
            )))
        }

        async fn validate_config(&self) -> Result<bool> {
            Ok(true)
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    // Mock translator that always succeeds
    struct SuccessTranslator {
        name: String,
    }

    #[async_trait::async_trait]
    impl Translator for SuccessTranslator {
        async fn translate(
            &self,
            text: &str,
            _source_lang: &str,
            _target_lang: &str,
        ) -> Result<String> {
            Ok(format!("[{}] {}", self.name, text))
        }

        async fn validate_config(&self) -> Result<bool> {
            Ok(true)
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    #[tokio::test]
    async fn test_fallback_on_failure() {
        // Create translation manager
        let mut manager = TranslationManager::new();

        // Register engines
        let failing_engine = Arc::new(FailingTranslator {
            name: "openai".to_string(),
        });
        let fallback_engine = Arc::new(SuccessTranslator {
            name: "ctranslate2".to_string(),
        });

        manager.register_engine(failing_engine);
        manager.register_engine(fallback_engine);

        // Set active and fallback engines
        manager.set_active_engine("openai").await.unwrap();
        manager.set_fallback_engine("ctranslate2").await.unwrap();

        // Track notifications
        let notifications = Arc::new(Mutex::new(Vec::new()));
        let notifications_clone = notifications.clone();

        manager
            .set_error_callback(Arc::new(move |notification| {
                let notifications = notifications_clone.clone();
                tokio::spawn(async move {
                    notifications.lock().await.push(notification);
                });
            }))
            .await;

        // Attempt translation - should fallback
        let result = manager
            .translate("Hello", "en", "ja")
            .await
            .expect("Translation should succeed with fallback");

        assert_eq!(result, "[ctranslate2] Hello");

        // Give callback time to execute
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Verify notification was sent
        let notifs = notifications.lock().await;
        assert_eq!(notifs.len(), 1);
        assert_eq!(notifs[0].failed_engine, "openai");
        assert_eq!(notifs[0].fallback_engine, "ctranslate2");
        assert!(notifs[0].fallback_succeeded);
    }

    #[tokio::test]
    async fn test_no_fallback_when_same_engine() {
        // Create translation manager
        let mut manager = TranslationManager::new();

        // Register engine
        let failing_engine = Arc::new(FailingTranslator {
            name: "openai".to_string(),
        });

        manager.register_engine(failing_engine);

        // Set both active and fallback to same engine
        manager.set_active_engine("openai").await.unwrap();
        manager.set_fallback_engine("openai").await.unwrap();

        // Track notifications
        let notifications = Arc::new(Mutex::new(Vec::new()));
        let notifications_clone = notifications.clone();

        manager
            .set_error_callback(Arc::new(move |notification| {
                let notifications = notifications_clone.clone();
                tokio::spawn(async move {
                    notifications.lock().await.push(notification);
                });
            }))
            .await;

        // Attempt translation - should fail without fallback
        let result = manager.translate("Hello", "en", "ja").await;

        assert!(result.is_err());

        // Give callback time to execute
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Verify no notification was sent (since fallback wasn't attempted)
        let notifs = notifications.lock().await;
        assert_eq!(notifs.len(), 0);
    }

    #[tokio::test]
    async fn test_fallback_also_fails() {
        // Create translation manager
        let mut manager = TranslationManager::new();

        // Register engines - both fail
        let failing_engine1 = Arc::new(FailingTranslator {
            name: "openai".to_string(),
        });
        let failing_engine2 = Arc::new(FailingTranslator {
            name: "ctranslate2".to_string(),
        });

        manager.register_engine(failing_engine1);
        manager.register_engine(failing_engine2);

        // Set active and fallback engines
        manager.set_active_engine("openai").await.unwrap();
        manager.set_fallback_engine("ctranslate2").await.unwrap();

        // Track notifications
        let notifications = Arc::new(Mutex::new(Vec::new()));
        let notifications_clone = notifications.clone();

        manager
            .set_error_callback(Arc::new(move |notification| {
                let notifications = notifications_clone.clone();
                tokio::spawn(async move {
                    notifications.lock().await.push(notification);
                });
            }))
            .await;

        // Attempt translation - should fail
        let result = manager.translate("Hello", "en", "ja").await;

        assert!(result.is_err());

        // Give callback time to execute
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Verify notification was sent with fallback_succeeded = false
        let notifs = notifications.lock().await;
        assert_eq!(notifs.len(), 1);
        assert_eq!(notifs[0].failed_engine, "openai");
        assert_eq!(notifs[0].fallback_engine, "ctranslate2");
        assert!(!notifs[0].fallback_succeeded);
    }

    #[tokio::test]
    async fn test_no_fallback_when_primary_succeeds() {
        // Create translation manager
        let mut manager = TranslationManager::new();

        // Register engines
        let success_engine = Arc::new(SuccessTranslator {
            name: "openai".to_string(),
        });
        let fallback_engine = Arc::new(SuccessTranslator {
            name: "ctranslate2".to_string(),
        });

        manager.register_engine(success_engine);
        manager.register_engine(fallback_engine);

        // Set active and fallback engines
        manager.set_active_engine("openai").await.unwrap();
        manager.set_fallback_engine("ctranslate2").await.unwrap();

        // Track notifications
        let notifications = Arc::new(Mutex::new(Vec::new()));
        let notifications_clone = notifications.clone();

        manager
            .set_error_callback(Arc::new(move |notification| {
                let notifications = notifications_clone.clone();
                tokio::spawn(async move {
                    notifications.lock().await.push(notification);
                });
            }))
            .await;

        // Attempt translation - should succeed with primary
        let result = manager
            .translate("Hello", "en", "ja")
            .await
            .expect("Translation should succeed");

        assert_eq!(result, "[openai] Hello");

        // Give callback time to execute
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Verify no notification was sent (since primary succeeded)
        let notifs = notifications.lock().await;
        assert_eq!(notifs.len(), 0);
    }
}
