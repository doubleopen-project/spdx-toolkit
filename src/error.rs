// SPDX-FileCopyrightText: 2021 HH Partners
//
// SPDX-License-Identifier: MIT

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("error with json")]
    SerdeJson {
        #[from]
        source: serde_json::Error,
    },

    #[error("error with http request")]
    Request {
        #[from]
        source: reqwest::Error,
    },

    #[error("error with graph: {0}")]
    Graph(String),
}
