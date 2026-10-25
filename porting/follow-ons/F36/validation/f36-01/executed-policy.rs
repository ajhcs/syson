// SPDX-License-Identifier: EPL-2.0
//! Focus requests for the bounded staged inspector; model history stays separate.
#[derive(Default)]
pub(crate) struct InspectorAccessibility {
    return_to: Option<String>,
    focus: Option<String>,
}
impl InspectorAccessibility {
    pub fn begin(&mut self) {
        self.focus = None;
    }
    pub fn finish(&mut self, action: &str, success: bool, attribute: &str, rename: bool) {
        if !success {
            return;
        }
        self.focus = match action {
            "select" => Some(
                if rename {
                    "rename"
                } else {
                    "selection-heading"
                }
                .into(),
            ),
            "stage" | "rename" | "attribute" => {
                self.return_to = Some(match action {
                    "stage" => "name".into(),
                    "rename" => "rename".into(),
                    _ => format!("scalar-{attribute}"),
                });
                Some("commit-staged".into())
            }
            "commit" | "cancel" => self.return_to.take(),
            "import" => {
                self.return_to = None;
                Some("kind".into())
            }
            _ => None,
        };
    }
    pub fn focus(&self) -> &str {
        self.focus.as_deref().unwrap_or("")
    }
}
