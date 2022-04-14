// SPDX-FileCopyrightText: 2020 HH Partners
//
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LicenseList {
    pub license_list_version: String,
    #[serde(default)]
    pub licenses: Vec<License>,
    #[serde(default)]
    pub exceptions: Vec<Exception>,
    pub release_date: String,
}

impl LicenseList {
    /// Get [`LicenseList`] from GitHub. Specify version as `Some("v3.14")` or use None for the
    /// latest version.
    ///
    /// # Errors
    ///
    /// Returns [`SpdxError`] if there is a problem with retrieving the license list from GitHub
    /// or if deserializing the data fails.
    pub fn from_github(version: Option<&str>) -> Result<Self, Error> {
        let version = version.unwrap_or("master");

        let licenses_url = format!(
            "https://raw.githubusercontent.com/spdx/license-list-data/{version}/json/licenses.json"
        );
        let body = reqwest::blocking::get(licenses_url)?.text()?;
        let mut license_list: Self = serde_json::from_str(&body)?;

        let exceptions_url =
            format!("https://raw.githubusercontent.com/spdx/license-list-data/{version}/json/exceptions.json");
        let body = reqwest::blocking::get(exceptions_url)?.text()?;
        let exceptions_list: Self = serde_json::from_str(&body)?;
        license_list.exceptions = exceptions_list.exceptions;
        Ok(license_list)
    }

    pub fn includes_license(&self, spdx_id: &str) -> bool {
        self.licenses
            .iter()
            .any(|license| license.license_id == spdx_id)
    }

    pub fn includes_exception(&self, exception_id: &str) -> bool {
        self.exceptions
            .iter()
            .any(|exception| exception.license_exception_id == exception_id)
    }

    #[allow(clippy::doc_markdown)]
    /// Return true if the input expression is a license on the SPDX license list, a LicenseRef
    /// license, a `DocumentRef` license, `NONE` or `NOASSERTION`.
    pub fn is_valid_license(&self, expression: &str) -> bool {
        expression == "NOASSERTION"
            || expression == "NONE"
            || expression.starts_with("LicenseRef-")
            || expression.starts_with("DocumentRef-")
            || self.includes_license(&expression.replace('+', ""))
            || self.includes_exception(expression)
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct License {
    pub reference: String,
    pub is_deprecated_license_id: bool,
    pub details_url: String,
    pub reference_number: i32,
    pub name: String,
    pub license_id: String,
    pub see_also: Vec<String>,
    pub is_osi_approved: bool,
    #[serde(default)]
    pub is_fsf_libre: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Exception {
    pub reference: String,
    pub is_deprecated_license_id: bool,
    pub details_url: String,
    pub reference_number: i32,
    pub name: String,
    pub license_exception_id: String,
    pub see_also: Vec<String>,
}

#[cfg(test)]
mod test_license_list {
    use std::fs::read_to_string;

    use super::*;

    #[test]
    fn licenses_deserialization_works() {
        let licenses_file =
            read_to_string("tests/data/licenses.json").expect("Should always exist");
        let _license_list: LicenseList =
            serde_json::from_str(&licenses_file).expect("Deseralization should work.");
    }

    #[test]
    fn exceptions_deserialization_works() {
        let licenses_file =
            read_to_string("tests/data/exceptions.json").expect("Should always exist");
        let _license_list: LicenseList =
            serde_json::from_str(&licenses_file).expect("Deseralization should work.");
    }

    #[test]
    fn from_github_works() {
        let license_list = LicenseList::from_github(None).unwrap();

        assert!(!license_list.licenses.is_empty());
        assert!(!license_list.exceptions.is_empty());
    }

    #[test]
    fn correctly_get_older_version_from_github() {
        let license_list = LicenseList::from_github(Some("v3.14")).unwrap();

        assert!(!license_list.licenses.is_empty());
        assert!(!license_list.exceptions.is_empty());

        assert_eq!(license_list.license_list_version, "3.14".to_string());
    }

    #[test]
    fn bsd_works() {
        let license_list = LicenseList::from_github(None).unwrap();

        assert!(!license_list.includes_license("BSD"));
        assert!(!license_list.includes_exception("BSD"));
    }

    #[test]
    fn correctly_determine_validity_of_licenses() {
        let license_list = LicenseList::from_github(None).unwrap();
        assert!(license_list.is_valid_license("MIT"));
        assert!(license_list.is_valid_license("GPL-2.0-or-later"));
        assert!(!license_list.is_valid_license("invalid-license"));
        assert!(license_list.is_valid_license("LicenseRef-license1"));
        assert!(license_list.is_valid_license("DocumentRef-document:LicenseRef-license1"));
        assert!(license_list.is_valid_license("NONE"));
        assert!(license_list.is_valid_license("NOASSERTION"));
    }
}
