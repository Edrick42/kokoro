use leptos::prelude::*;
use crate::server::auth::Register;

#[component]
pub fn RegisterPage() -> impl IntoView {
    let register_action = ServerAction::<Register>::new();
    let value = register_action.value();

    // On successful register, store session and redirect
    Effect::new(move || {
        if let Some(Ok(_session)) = value.get() {
            #[cfg(feature = "hydrate")]
            {
                if let Some(storage) = web_sys::window()
                    .and_then(|w| w.local_storage().ok())
                    .flatten()
                {
                    let _ = storage.set_item("kokoro_token", &_session.token);
                    let _ = storage.set_item("kokoro_user", &_session.display_name);
                    let _ = storage.set_item("kokoro_email", &_session.email);
                }
                if let Some(window) = web_sys::window() {
                    let _ = window.location().set_href("/profile");
                }
            }
        }
    });

    let error_msg = move || {
        value.get().and_then(|r: Result<_, _>| r.err()).map(|e| e.to_string())
    };

    view! {
        <div class="auth-form">
            <h2>"Register"</h2>

            {move || error_msg().map(|msg| view! {
                <p class="error-msg">{msg}</p>
            })}

            <ActionForm action=register_action>
                <div class="form-group">
                    <label>"Display Name"</label>
                    <input type="text" name="display_name" placeholder="Your name" required/>
                </div>

                <div class="form-group">
                    <label>"Email"</label>
                    <input type="email" name="email" placeholder="your@email.com" required/>
                </div>

                <div class="form-group">
                    <label>"Password"</label>
                    <input type="password" name="password" placeholder="Min 8 characters" required minlength="8"/>
                </div>

                <button type="submit" class="btn btn--primary" style="width:100%;">
                    {move || if register_action.pending().get() { "Creating account..." } else { "Create Account" }}
                </button>
            </ActionForm>

            <p style="text-align:center;margin-top:16px;font-size:12px;">
                "Already have an account? " <a href="/login">"Login"</a>
            </p>
        </div>
    }
}
