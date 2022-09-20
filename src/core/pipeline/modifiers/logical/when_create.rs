use async_trait::async_trait;
use crate::core::pipeline::modifier::Modifier;
use crate::core::pipeline::Pipeline;
use crate::core::pipeline::context::Context;
use crate::core::pipeline::context::Purpose::Create;

#[derive(Debug, Clone)]
pub struct WhenCreateModifier {
    pipeline: Pipeline
}

impl WhenCreateModifier {
    pub fn new(pipeline: Pipeline) -> Self {
        return WhenCreateModifier {
            pipeline
        };
    }
}

#[async_trait]
impl Modifier for WhenCreateModifier {

    fn name(&self) -> &'static str {
        "whenCreate"
    }

    async fn call(&self, ctx: Context) -> Context {
        if ctx.purpose == Create {
            self.pipeline.process(ctx.clone()).await
        } else {
            ctx
        }
    }
}