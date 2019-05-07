// Copyright 2019 Normation SAS
//
// This file is part of Rudder.
//
// Rudder is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// In accordance with the terms of section 7 (7. Additional Terms.) of
// the GNU General Public License version 3, the copyright holders add
// the following Additional permissions:
// Notwithstanding to the terms of section 5 (5. Conveying Modified Source
// Versions) and 6 (6. Conveying Non-Source Forms.) of the GNU General
// Public License version 3, when you create a Related Module, this
// Related Module is not considered as a part of the work and may be
// distributed under the license agreement of your choice.
// A "Related Module" means a set of sources files including their
// documentation that, without modification of the Source Code, enables
// supplementary functions or services in addition to those offered by
// the Software.
//
// Rudder is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Rudder.  If not, see <http://www.gnu.org/licenses/>.

pub mod watch;

use crate::configuration::LogComponent;
use crate::error::Error;
use crate::processing::ReceivedFile;
use flate2::read::GzDecoder;
use openssl::pkcs7::Pkcs7;
use openssl::pkcs7::Pkcs7Flags;
use openssl::stack::Stack;
use openssl::x509::store::X509Store;
use openssl::x509::store::X509StoreBuilder;
use openssl::x509::X509;
use slog::slog_debug;
use slog_scope::debug;
use std::ffi::OsStr;
use std::io::Read;

pub fn read_file_content(path: &ReceivedFile) -> Result<String, Error> {
    debug!("Reading {:#?} content", path);
    let data = std::fs::read(path)?;

    Ok(match path.extension().map(OsStr::to_str) {
        Some(Some("gz")) => {
            debug!("{:?} has .gz extension, extracting", path; "component" => LogComponent::Watcher);
            let mut gz = GzDecoder::new(data.as_slice());
            let mut s = String::new();
            gz.read_to_string(&mut s)?;
            s
        }
        // Let's assume everything else in this directory is a text file
        _ => {
            debug!("{:?} has no gz/xz extension, no extraction needed", path; "component" => LogComponent::Watcher);
            String::from_utf8(data)?
        }
    })
}

// SMIME content and node certificate
// impl NodeCertificate (X509 store par machine?)
// FIXME check certs are individually compared

// DETACHED??? -detached -> non
// valider le bon node

pub fn signature(input: &[u8], certs: &Stack<X509>, store: &X509Store) -> Result<String, Error> {
    let (signature, content) = Pkcs7::from_smime(input)?;

    signature.verify(
        // No need for certs as all our nodes certs are individually trusted in store?
        certs,
        store,
        Some(&content.clone().unwrap()),
        // TODO use out buffer or content directly?
        None,
        Pkcs7Flags::empty(),
    )?;

    Ok(String::from_utf8(content.expect("empty signed message"))?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::{fs::read_to_string, str::FromStr};

    #[test]
    fn it_reads_gzipped_files() {
        let reference = read_to_string("tests/test_gz/normal.log").unwrap();
        assert_eq!(
            read_file_content(&PathBuf::from_str("tests/test_gz/normal.log.gz").unwrap()).unwrap(),
            reference
        );
    }

    #[test]
    fn it_reads_plain_files() {
        let reference = read_to_string("tests/test_gz/normal.log").unwrap();
        assert_eq!(
            read_file_content(&PathBuf::from_str("tests/test_gz/normal.log").unwrap()).unwrap(),
            reference
        );
    }

    #[test]
    fn it_reads_signed_content() {
        let reference = read_to_string("tests/test_smime/normal.log").unwrap();

        let x509 = X509::from_pem(
            read_file_content(&PathBuf::from_str("tests/keys/localhost.cert").unwrap())
                .unwrap()
                .as_bytes(),
        )
        .unwrap();
        let x509bis = X509::from_pem(
            read_file_content(&PathBuf::from_str("tests/keys/localhost2.cert").unwrap())
                .unwrap()
                .as_bytes(),
        )
        .unwrap();

        // Store
        let mut builder = X509StoreBuilder::new().unwrap();
        builder.add_cert(x509.clone()).unwrap();
        builder.add_cert(x509bis.clone()).unwrap();
        let store = builder.build();

        // Certs
        let mut certs = Stack::new().unwrap();
        certs.push(x509bis).unwrap();

        assert_eq!(
            // openssl smime -sign -signer ../keys/localhost.cert -in normal.log
            //         -out normal.signed -inkey ../keys/localhost.priv -passin "pass:Cfengine passphrase"
            signature(
                read_file_content(&PathBuf::from_str("tests/test_smime/normal.signed").unwrap())
                    .unwrap()
                    .as_bytes(),
                &certs,
                &store,
            )
            .unwrap(),
            reference
        );
    }
}
