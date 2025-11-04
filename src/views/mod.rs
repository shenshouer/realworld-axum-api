use askama::Template;
use axum::{
    extract::{Path, Query},
    response::{Html, IntoResponse, Redirect},
};
use serde::Deserialize;

use crate::errors::{AppError, Lang};

pub async fn start_handler() -> Redirect {
    Redirect::temporary("/en/index.html")
}

/// This type collects the query parameter `?name=` (if present)
#[derive(Debug, Deserialize)]
pub struct IndexHandlerQuery {
    #[serde(default)]
    name: String,
}

/// This is the first localized page your user sees.
///
/// It has arguments in the path that need to be parsable using `serde::Deserialize`; see `Lang`
/// for an explanation. And also query parameters (anything after `?` in the incoming URL).
pub async fn index_handler(
    Path((lang,)): Path<(Lang,)>,
    Query(query): Query<IndexHandlerQuery>,
) -> Result<impl IntoResponse, AppError> {
    // In the template we both use `{% match lang %}` and `{% if lang !=`, the former to select the
    // text of a specific language, e.g. in the `<title>`; and the latter to display references to
    // all other available languages except the currently selected one.
    // The field `name` will contain the value of the query parameter of the same name.
    // In `IndexHandlerQuery` we annotated the field with `#[serde(default)]`, so if the value is
    // absent, an empty string is selected by default, which is visible to the user an empty
    // `<input type="text" />` element.
    #[derive(Debug, Template)]
    #[template(path = "index.askama")]
    struct Tmpl {
        lang: Lang,
        name: String,
    }

    let template = Tmpl {
        lang,
        name: query.name,
    };
    Ok(Html(template.render()?))
}

#[derive(Debug, Deserialize)]
pub struct GreetingHandlerQuery {
    name: String,
}

/// This is the final page of this example application.
///
/// Like `index_handler` it contains a language in the URL, and a query parameter to read the user's
/// provided name. In here, the query argument `name` has no default value, so axum will show
/// an error message if absent.
pub async fn greeting_handler(
    Path((lang,)): Path<(Lang,)>,
    Query(query): Query<GreetingHandlerQuery>,
) -> Result<impl IntoResponse, AppError> {
    #[derive(Debug, Template)]
    #[template(path = "greet.askama")]
    struct Tmpl {
        lang: Lang,
        name: String,
    }

    let template = Tmpl {
        lang,
        name: query.name,
    };
    Ok(Html(template.render()?))
}
