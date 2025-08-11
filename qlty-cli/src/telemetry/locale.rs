// From https://github.com/i509VCB/current_locale
//
// The MIT License (MIT)
//
// Copyright (c) 2021 i509VCB
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
pub fn current_locale() -> String {
    let lang_env = std::env::var("LANG");

    match lang_env {
        Ok(raw) => {
            // Unset locale - C ANSI standards say default to en-US
            if raw == "C" {
                "en-US".to_owned()
            } else {
                // Find one of the following to split off the lang code:
                // First index of `.` as in `en_US.UTF_8`
                // A space which separates generic code from char set.
                // Terminate at an `@` which specifies a locale at a specific location
                if let Some(pos) = raw.find([' ', '.']) {
                    let (raw_lang_code, _) = raw.split_at(pos);
                    let result = raw_lang_code.replace('_', "-");

                    // Finally replace underscores with `-` and drop everything after an `@`
                    result.split('@').next().unwrap().to_string()
                } else {
                    // LANG is not IETF compliant
                    "".to_owned()
                }
            }
        }
        Err(_) => "".to_owned(),
    }
}
