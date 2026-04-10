use leptos::prelude::*;

#[component]
pub fn LoginPage() -> impl IntoView {
    view! {
        <div class="auth-form">
            <h2>"Login"</h2>

            <div class="form-group">
                <label>"Email"</label>
                <input type="email" placeholder="your@email.com"/>
            </div>

            <div class="form-group">
                <label>"Password"</label>
                <input type="password" placeholder="********"/>
            </div>

            <button class="btn btn--primary" style="width:100%;">"Login"</button>

            <p style="text-align:center;margin-top:16px;font-size:12px;">
                "No account? " <a href="/register">"Register"</a>
            </p>
        </div>
    }
}
