//! Root application component with router.

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::*;
use leptos_router::path;

use crate::components::nav::Nav;
use crate::components::footer::Footer;
use crate::pages;

/// Root component — wraps everything in router + meta providers.
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/kokoro-ui.css"/>
        <Title text="Kokoro — Virtual Pet Bio-Simulation"/>
        <Meta name="viewport" content="width=device-width, initial-scale=1.0"/>

        <Nav/>

        <main class="container page">
            <Router>
                <Routes fallback=|| view! { <p>"Page not found."</p> }>
                    <Route path=path!("/") view=pages::home::HomePage/>
                    <Route path=path!("/species") view=pages::species_list::SpeciesListPage/>
                    <Route path=path!("/species/:name") view=pages::species_detail::SpeciesDetailPage/>
                    <Route path=path!("/login") view=pages::login::LoginPage/>
                    <Route path=path!("/register") view=pages::register::RegisterPage/>
                </Routes>
            </Router>
        </main>

        <Footer/>
    }
}
