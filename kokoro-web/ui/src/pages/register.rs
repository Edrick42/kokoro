use leptos::prelude::*;

#[component]
pub fn RegisterPage() -> impl IntoView {
    view! {
        <div class="auth-form">
            <h2>"Register"</h2>

            <div class="form-group">
                <label>"Display Name"</label>
                <input type="text" placeholder="Your name"/>
            </div>

            <div class="form-group">
                <label>"Email"</label>
                <input type="email" placeholder="your@email.com"/>
            </div>

            <div class="form-group">
                <label>"Password"</label>
                <input type="password" placeholder="Min 8 characters"/>
            </div>

            <button class="btn btn--primary" style="width:100%;">"Create Account"</button>

            <p style="text-align:center;margin-top:16px;font-size:12px;">
                "Already have an account? " <a href="/login">"Login"</a>
            </p>
        </div>
    }
}
