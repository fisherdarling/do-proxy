/// Generates worker-rs [`worker::DurableObject`] glue code for a type that impls [`crate::DoProxy`].
#[macro_export]
macro_rules! do_proxy {
    ($proxy_name:ident, $obj_name:ident) => {
        $crate::paste::paste! {
            mod [<__ $obj_name:camel>] {
                use super::$proxy_name;

                use $crate::{
                    worker::*,
                    ProxiedRequest,
                    DoProxy,
                };

                #[worker::durable_object]
                pub struct $obj_name {
                    state: State,
                    env: Env,
                    proxy: Option<$proxy_name>,
                }

                #[worker::durable_object]
                impl worker::DurableObject for $obj_name {
                    fn new(state: State, env: Env) -> Self {
                        Self {
                            state,
                            env,
                            proxy: None,
                        }
                    }

                    async fn fetch(&mut self, req: worker::Request) -> worker::Result<Response> {
                        let mut ctx = $crate::Ctx::new(&self.state, &self.env);
                        $proxy_name::run_request(&mut self.proxy, &mut ctx, Some(req)).await
                    }

                    async fn alarm(&mut self) -> worker::Result<Response> {
                        let mut ctx = $crate::Ctx::new(&self.state, &self.env);
                        $proxy_name::run_request(&mut self.proxy, &mut ctx, None).await
                    }
                }
            }
        }
    };
}
