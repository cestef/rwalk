use std::sync::Arc;

use super::response_filter;
use crate::RwalkError;
use rhai::{Engine, Scope, AST};

response_filter!(
    ScriptFilter,
    (AST, Arc<Engine>),
    needs_body = true,
    |res: &RwalkResponse, (ast, engine): &(AST, Arc<Engine>)| {
        let mut scope = Scope::new();
        scope.push("response", res.clone());
        let result = engine.eval_ast_with_scope::<bool>(&mut scope, ast);
        match result {
            Ok(result) => Ok(result),
            Err(e) => Err(RwalkError::RhaiError(e.to_string())),
        }
    },
    "script",
    ["rhai"],
    transform = |arg: String| -> Result<(AST, Arc<Engine>)> {
        let mut engine = Engine::new();
        engine.build_type::<RwalkResponse>();
        let contents = std::fs::read_to_string(&arg)?;
        let ast = engine
            .compile(&contents)
            .map_err(|e| RwalkError::RhaiError(e.to_string()))?;

        Ok((ast, Arc::new(engine)))
    }
);
