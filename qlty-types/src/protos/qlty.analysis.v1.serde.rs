// @generated
impl serde::Serialize for AnalysisResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "ANALYSIS_RESULT_UNSPECIFIED",
            Self::Success => "ANALYSIS_RESULT_SUCCESS",
            Self::Error => "ANALYSIS_RESULT_ERROR",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for AnalysisResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "ANALYSIS_RESULT_UNSPECIFIED",
            "ANALYSIS_RESULT_SUCCESS",
            "ANALYSIS_RESULT_ERROR",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AnalysisResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "ANALYSIS_RESULT_UNSPECIFIED" => Ok(AnalysisResult::Unspecified),
                    "ANALYSIS_RESULT_SUCCESS" => Ok(AnalysisResult::Success),
                    "ANALYSIS_RESULT_ERROR" => Ok(AnalysisResult::Error),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for Category {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "CATEGORY_UNSPECIFIED",
            Self::Bug => "CATEGORY_BUG",
            Self::Vulnerability => "CATEGORY_VULNERABILITY",
            Self::Structure => "CATEGORY_STRUCTURE",
            Self::Duplication => "CATEGORY_DUPLICATION",
            Self::SecurityHotspot => "CATEGORY_SECURITY_HOTSPOT",
            Self::Performance => "CATEGORY_PERFORMANCE",
            Self::Documentation => "CATEGORY_DOCUMENTATION",
            Self::TypeCheck => "CATEGORY_TYPE_CHECK",
            Self::Style => "CATEGORY_STYLE",
            Self::AntiPattern => "CATEGORY_ANTI_PATTERN",
            Self::Accessibility => "CATEGORY_ACCESSIBILITY",
            Self::DeadCode => "CATEGORY_DEAD_CODE",
            Self::Lint => "CATEGORY_LINT",
            Self::Secret => "CATEGORY_SECRET",
            Self::DependencyAlert => "CATEGORY_DEPENDENCY_ALERT",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for Category {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "CATEGORY_UNSPECIFIED",
            "CATEGORY_BUG",
            "CATEGORY_VULNERABILITY",
            "CATEGORY_STRUCTURE",
            "CATEGORY_DUPLICATION",
            "CATEGORY_SECURITY_HOTSPOT",
            "CATEGORY_PERFORMANCE",
            "CATEGORY_DOCUMENTATION",
            "CATEGORY_TYPE_CHECK",
            "CATEGORY_STYLE",
            "CATEGORY_ANTI_PATTERN",
            "CATEGORY_ACCESSIBILITY",
            "CATEGORY_DEAD_CODE",
            "CATEGORY_LINT",
            "CATEGORY_SECRET",
            "CATEGORY_DEPENDENCY_ALERT",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Category;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "CATEGORY_UNSPECIFIED" => Ok(Category::Unspecified),
                    "CATEGORY_BUG" => Ok(Category::Bug),
                    "CATEGORY_VULNERABILITY" => Ok(Category::Vulnerability),
                    "CATEGORY_STRUCTURE" => Ok(Category::Structure),
                    "CATEGORY_DUPLICATION" => Ok(Category::Duplication),
                    "CATEGORY_SECURITY_HOTSPOT" => Ok(Category::SecurityHotspot),
                    "CATEGORY_PERFORMANCE" => Ok(Category::Performance),
                    "CATEGORY_DOCUMENTATION" => Ok(Category::Documentation),
                    "CATEGORY_TYPE_CHECK" => Ok(Category::TypeCheck),
                    "CATEGORY_STYLE" => Ok(Category::Style),
                    "CATEGORY_ANTI_PATTERN" => Ok(Category::AntiPattern),
                    "CATEGORY_ACCESSIBILITY" => Ok(Category::Accessibility),
                    "CATEGORY_DEAD_CODE" => Ok(Category::DeadCode),
                    "CATEGORY_LINT" => Ok(Category::Lint),
                    "CATEGORY_SECRET" => Ok(Category::Secret),
                    "CATEGORY_DEPENDENCY_ALERT" => Ok(Category::DependencyAlert),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for ComponentType {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "COMPONENT_TYPE_UNSPECIFIED",
            Self::File => "COMPONENT_TYPE_FILE",
            Self::Directory => "COMPONENT_TYPE_DIRECTORY",
            Self::Project => "COMPONENT_TYPE_PROJECT",
            Self::Module => "COMPONENT_TYPE_MODULE",
            Self::Class => "COMPONENT_TYPE_CLASS",
            Self::Function => "COMPONENT_TYPE_FUNCTION",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for ComponentType {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "COMPONENT_TYPE_UNSPECIFIED",
            "COMPONENT_TYPE_FILE",
            "COMPONENT_TYPE_DIRECTORY",
            "COMPONENT_TYPE_PROJECT",
            "COMPONENT_TYPE_MODULE",
            "COMPONENT_TYPE_CLASS",
            "COMPONENT_TYPE_FUNCTION",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ComponentType;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "COMPONENT_TYPE_UNSPECIFIED" => Ok(ComponentType::Unspecified),
                    "COMPONENT_TYPE_FILE" => Ok(ComponentType::File),
                    "COMPONENT_TYPE_DIRECTORY" => Ok(ComponentType::Directory),
                    "COMPONENT_TYPE_PROJECT" => Ok(ComponentType::Project),
                    "COMPONENT_TYPE_MODULE" => Ok(ComponentType::Module),
                    "COMPONENT_TYPE_CLASS" => Ok(ComponentType::Class),
                    "COMPONENT_TYPE_FUNCTION" => Ok(ComponentType::Function),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for ExecutionVerb {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "EXECUTION_VERB_UNSPECIFIED",
            Self::Check => "EXECUTION_VERB_CHECK",
            Self::Fmt => "EXECUTION_VERB_FMT",
            Self::Validate => "EXECUTION_VERB_VALIDATE",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for ExecutionVerb {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "EXECUTION_VERB_UNSPECIFIED",
            "EXECUTION_VERB_CHECK",
            "EXECUTION_VERB_FMT",
            "EXECUTION_VERB_VALIDATE",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ExecutionVerb;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "EXECUTION_VERB_UNSPECIFIED" => Ok(ExecutionVerb::Unspecified),
                    "EXECUTION_VERB_CHECK" => Ok(ExecutionVerb::Check),
                    "EXECUTION_VERB_FMT" => Ok(ExecutionVerb::Fmt),
                    "EXECUTION_VERB_VALIDATE" => Ok(ExecutionVerb::Validate),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for ExitResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "EXIT_RESULT_UNSPECIFIED",
            Self::Success => "EXIT_RESULT_SUCCESS",
            Self::KnownError => "EXIT_RESULT_KNOWN_ERROR",
            Self::UnknownError => "EXIT_RESULT_UNKNOWN_ERROR",
            Self::NoIssues => "EXIT_RESULT_NO_ISSUES",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for ExitResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "EXIT_RESULT_UNSPECIFIED",
            "EXIT_RESULT_SUCCESS",
            "EXIT_RESULT_KNOWN_ERROR",
            "EXIT_RESULT_UNKNOWN_ERROR",
            "EXIT_RESULT_NO_ISSUES",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ExitResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "EXIT_RESULT_UNSPECIFIED" => Ok(ExitResult::Unspecified),
                    "EXIT_RESULT_SUCCESS" => Ok(ExitResult::Success),
                    "EXIT_RESULT_KNOWN_ERROR" => Ok(ExitResult::KnownError),
                    "EXIT_RESULT_UNKNOWN_ERROR" => Ok(ExitResult::UnknownError),
                    "EXIT_RESULT_NO_ISSUES" => Ok(ExitResult::NoIssues),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for Invocation {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.workspace_id.is_empty() {
            len += 1;
        }
        if !self.project_id.is_empty() {
            len += 1;
        }
        if !self.reference.is_empty() {
            len += 1;
        }
        if !self.build_id.is_empty() {
            len += 1;
        }
        if self.build_timestamp.is_some() {
            len += 1;
        }
        if !self.commit_sha.is_empty() {
            len += 1;
        }
        if !self.id.is_empty() {
            len += 1;
        }
        if !self.qlty_cli_version.is_empty() {
            len += 1;
        }
        if !self.plugin_name.is_empty() {
            len += 1;
        }
        if !self.driver_name.is_empty() {
            len += 1;
        }
        if !self.prefix.is_empty() {
            len += 1;
        }
        if !self.plugin_version.is_empty() {
            len += 1;
        }
        if self.verb != 0 {
            len += 1;
        }
        if self.targets_count != 0 {
            len += 1;
        }
        if !self.target_paths.is_empty() {
            len += 1;
        }
        if !self.config_paths.is_empty() {
            len += 1;
        }
        if !self.script.is_empty() {
            len += 1;
        }
        if !self.cwd.is_empty() {
            len += 1;
        }
        if !self.env.is_empty() {
            len += 1;
        }
        if self.started_at.is_some() {
            len += 1;
        }
        if self.duration_secs != 0. {
            len += 1;
        }
        if self.exit_code.is_some() {
            len += 1;
        }
        if !self.stdout.is_empty() {
            len += 1;
        }
        if !self.stderr.is_empty() {
            len += 1;
        }
        if self.tmpfile_path.is_some() {
            len += 1;
        }
        if self.tmpfile_contents.is_some() {
            len += 1;
        }
        if self.exit_result != 0 {
            len += 1;
        }
        if self.parser_error.is_some() {
            len += 1;
        }
        if self.issues_count != 0 {
            len += 1;
        }
        if self.rewrites_count != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("qlty.analysis.v1.Invocation", len)?;
        if !self.workspace_id.is_empty() {
            struct_ser.serialize_field("workspaceId", &self.workspace_id)?;
        }
        if !self.project_id.is_empty() {
            struct_ser.serialize_field("projectId", &self.project_id)?;
        }
        if !self.reference.is_empty() {
            struct_ser.serialize_field("reference", &self.reference)?;
        }
        if !self.build_id.is_empty() {
            struct_ser.serialize_field("buildId", &self.build_id)?;
        }
        if let Some(v) = self.build_timestamp.as_ref() {
            struct_ser.serialize_field("buildTimestamp", v)?;
        }
        if !self.commit_sha.is_empty() {
            struct_ser.serialize_field("commitSha", &self.commit_sha)?;
        }
        if !self.id.is_empty() {
            struct_ser.serialize_field("id", &self.id)?;
        }
        if !self.qlty_cli_version.is_empty() {
            struct_ser.serialize_field("qltyCliVersion", &self.qlty_cli_version)?;
        }
        if !self.plugin_name.is_empty() {
            struct_ser.serialize_field("pluginName", &self.plugin_name)?;
        }
        if !self.driver_name.is_empty() {
            struct_ser.serialize_field("driverName", &self.driver_name)?;
        }
        if !self.prefix.is_empty() {
            struct_ser.serialize_field("prefix", &self.prefix)?;
        }
        if !self.plugin_version.is_empty() {
            struct_ser.serialize_field("pluginVersion", &self.plugin_version)?;
        }
        if self.verb != 0 {
            let v = ExecutionVerb::try_from(self.verb)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.verb)))?;
            struct_ser.serialize_field("verb", &v)?;
        }
        if self.targets_count != 0 {
            struct_ser.serialize_field("targetsCount", &self.targets_count)?;
        }
        if !self.target_paths.is_empty() {
            struct_ser.serialize_field("targetPaths", &self.target_paths)?;
        }
        if !self.config_paths.is_empty() {
            struct_ser.serialize_field("configPaths", &self.config_paths)?;
        }
        if !self.script.is_empty() {
            struct_ser.serialize_field("script", &self.script)?;
        }
        if !self.cwd.is_empty() {
            struct_ser.serialize_field("cwd", &self.cwd)?;
        }
        if !self.env.is_empty() {
            struct_ser.serialize_field("env", &self.env)?;
        }
        if let Some(v) = self.started_at.as_ref() {
            struct_ser.serialize_field("startedAt", v)?;
        }
        if self.duration_secs != 0. {
            struct_ser.serialize_field("durationSecs", &self.duration_secs)?;
        }
        if let Some(v) = self.exit_code.as_ref() {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("exitCode", ToString::to_string(&v).as_str())?;
        }
        if !self.stdout.is_empty() {
            struct_ser.serialize_field("stdout", &self.stdout)?;
        }
        if !self.stderr.is_empty() {
            struct_ser.serialize_field("stderr", &self.stderr)?;
        }
        if let Some(v) = self.tmpfile_path.as_ref() {
            struct_ser.serialize_field("tmpfilePath", v)?;
        }
        if let Some(v) = self.tmpfile_contents.as_ref() {
            struct_ser.serialize_field("tmpfileContents", v)?;
        }
        if self.exit_result != 0 {
            let v = ExitResult::try_from(self.exit_result)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.exit_result)))?;
            struct_ser.serialize_field("exitResult", &v)?;
        }
        if let Some(v) = self.parser_error.as_ref() {
            struct_ser.serialize_field("parserError", v)?;
        }
        if self.issues_count != 0 {
            struct_ser.serialize_field("issuesCount", &self.issues_count)?;
        }
        if self.rewrites_count != 0 {
            struct_ser.serialize_field("rewritesCount", &self.rewrites_count)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Invocation {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "workspace_id",
            "workspaceId",
            "project_id",
            "projectId",
            "reference",
            "build_id",
            "buildId",
            "build_timestamp",
            "buildTimestamp",
            "commit_sha",
            "commitSha",
            "id",
            "qlty_cli_version",
            "qltyCliVersion",
            "plugin_name",
            "pluginName",
            "driver_name",
            "driverName",
            "prefix",
            "plugin_version",
            "pluginVersion",
            "verb",
            "targets_count",
            "targetsCount",
            "target_paths",
            "targetPaths",
            "config_paths",
            "configPaths",
            "script",
            "cwd",
            "env",
            "started_at",
            "startedAt",
            "duration_secs",
            "durationSecs",
            "exit_code",
            "exitCode",
            "stdout",
            "stderr",
            "tmpfile_path",
            "tmpfilePath",
            "tmpfile_contents",
            "tmpfileContents",
            "exit_result",
            "exitResult",
            "parser_error",
            "parserError",
            "issues_count",
            "issuesCount",
            "rewrites_count",
            "rewritesCount",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            WorkspaceId,
            ProjectId,
            Reference,
            BuildId,
            BuildTimestamp,
            CommitSha,
            Id,
            QltyCliVersion,
            PluginName,
            DriverName,
            Prefix,
            PluginVersion,
            Verb,
            TargetsCount,
            TargetPaths,
            ConfigPaths,
            Script,
            Cwd,
            Env,
            StartedAt,
            DurationSecs,
            ExitCode,
            Stdout,
            Stderr,
            TmpfilePath,
            TmpfileContents,
            ExitResult,
            ParserError,
            IssuesCount,
            RewritesCount,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "workspaceId" | "workspace_id" => Ok(GeneratedField::WorkspaceId),
                            "projectId" | "project_id" => Ok(GeneratedField::ProjectId),
                            "reference" => Ok(GeneratedField::Reference),
                            "buildId" | "build_id" => Ok(GeneratedField::BuildId),
                            "buildTimestamp" | "build_timestamp" => Ok(GeneratedField::BuildTimestamp),
                            "commitSha" | "commit_sha" => Ok(GeneratedField::CommitSha),
                            "id" => Ok(GeneratedField::Id),
                            "qltyCliVersion" | "qlty_cli_version" => Ok(GeneratedField::QltyCliVersion),
                            "pluginName" | "plugin_name" => Ok(GeneratedField::PluginName),
                            "driverName" | "driver_name" => Ok(GeneratedField::DriverName),
                            "prefix" => Ok(GeneratedField::Prefix),
                            "pluginVersion" | "plugin_version" => Ok(GeneratedField::PluginVersion),
                            "verb" => Ok(GeneratedField::Verb),
                            "targetsCount" | "targets_count" => Ok(GeneratedField::TargetsCount),
                            "targetPaths" | "target_paths" => Ok(GeneratedField::TargetPaths),
                            "configPaths" | "config_paths" => Ok(GeneratedField::ConfigPaths),
                            "script" => Ok(GeneratedField::Script),
                            "cwd" => Ok(GeneratedField::Cwd),
                            "env" => Ok(GeneratedField::Env),
                            "startedAt" | "started_at" => Ok(GeneratedField::StartedAt),
                            "durationSecs" | "duration_secs" => Ok(GeneratedField::DurationSecs),
                            "exitCode" | "exit_code" => Ok(GeneratedField::ExitCode),
                            "stdout" => Ok(GeneratedField::Stdout),
                            "stderr" => Ok(GeneratedField::Stderr),
                            "tmpfilePath" | "tmpfile_path" => Ok(GeneratedField::TmpfilePath),
                            "tmpfileContents" | "tmpfile_contents" => Ok(GeneratedField::TmpfileContents),
                            "exitResult" | "exit_result" => Ok(GeneratedField::ExitResult),
                            "parserError" | "parser_error" => Ok(GeneratedField::ParserError),
                            "issuesCount" | "issues_count" => Ok(GeneratedField::IssuesCount),
                            "rewritesCount" | "rewrites_count" => Ok(GeneratedField::RewritesCount),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Invocation;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct qlty.analysis.v1.Invocation")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Invocation, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut workspace_id__ = None;
                let mut project_id__ = None;
                let mut reference__ = None;
                let mut build_id__ = None;
                let mut build_timestamp__ = None;
                let mut commit_sha__ = None;
                let mut id__ = None;
                let mut qlty_cli_version__ = None;
                let mut plugin_name__ = None;
                let mut driver_name__ = None;
                let mut prefix__ = None;
                let mut plugin_version__ = None;
                let mut verb__ = None;
                let mut targets_count__ = None;
                let mut target_paths__ = None;
                let mut config_paths__ = None;
                let mut script__ = None;
                let mut cwd__ = None;
                let mut env__ = None;
                let mut started_at__ = None;
                let mut duration_secs__ = None;
                let mut exit_code__ = None;
                let mut stdout__ = None;
                let mut stderr__ = None;
                let mut tmpfile_path__ = None;
                let mut tmpfile_contents__ = None;
                let mut exit_result__ = None;
                let mut parser_error__ = None;
                let mut issues_count__ = None;
                let mut rewrites_count__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::WorkspaceId => {
                            if workspace_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("workspaceId"));
                            }
                            workspace_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ProjectId => {
                            if project_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("projectId"));
                            }
                            project_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Reference => {
                            if reference__.is_some() {
                                return Err(serde::de::Error::duplicate_field("reference"));
                            }
                            reference__ = Some(map_.next_value()?);
                        }
                        GeneratedField::BuildId => {
                            if build_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("buildId"));
                            }
                            build_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::BuildTimestamp => {
                            if build_timestamp__.is_some() {
                                return Err(serde::de::Error::duplicate_field("buildTimestamp"));
                            }
                            build_timestamp__ = map_.next_value()?;
                        }
                        GeneratedField::CommitSha => {
                            if commit_sha__.is_some() {
                                return Err(serde::de::Error::duplicate_field("commitSha"));
                            }
                            commit_sha__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Id => {
                            if id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::QltyCliVersion => {
                            if qlty_cli_version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("qltyCliVersion"));
                            }
                            qlty_cli_version__ = Some(map_.next_value()?);
                        }
                        GeneratedField::PluginName => {
                            if plugin_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pluginName"));
                            }
                            plugin_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::DriverName => {
                            if driver_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("driverName"));
                            }
                            driver_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Prefix => {
                            if prefix__.is_some() {
                                return Err(serde::de::Error::duplicate_field("prefix"));
                            }
                            prefix__ = Some(map_.next_value()?);
                        }
                        GeneratedField::PluginVersion => {
                            if plugin_version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pluginVersion"));
                            }
                            plugin_version__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Verb => {
                            if verb__.is_some() {
                                return Err(serde::de::Error::duplicate_field("verb"));
                            }
                            verb__ = Some(map_.next_value::<ExecutionVerb>()? as i32);
                        }
                        GeneratedField::TargetsCount => {
                            if targets_count__.is_some() {
                                return Err(serde::de::Error::duplicate_field("targetsCount"));
                            }
                            targets_count__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::TargetPaths => {
                            if target_paths__.is_some() {
                                return Err(serde::de::Error::duplicate_field("targetPaths"));
                            }
                            target_paths__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ConfigPaths => {
                            if config_paths__.is_some() {
                                return Err(serde::de::Error::duplicate_field("configPaths"));
                            }
                            config_paths__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Script => {
                            if script__.is_some() {
                                return Err(serde::de::Error::duplicate_field("script"));
                            }
                            script__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Cwd => {
                            if cwd__.is_some() {
                                return Err(serde::de::Error::duplicate_field("cwd"));
                            }
                            cwd__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Env => {
                            if env__.is_some() {
                                return Err(serde::de::Error::duplicate_field("env"));
                            }
                            env__ = Some(
                                map_.next_value::<std::collections::HashMap<_, _>>()?
                            );
                        }
                        GeneratedField::StartedAt => {
                            if started_at__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startedAt"));
                            }
                            started_at__ = map_.next_value()?;
                        }
                        GeneratedField::DurationSecs => {
                            if duration_secs__.is_some() {
                                return Err(serde::de::Error::duplicate_field("durationSecs"));
                            }
                            duration_secs__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ExitCode => {
                            if exit_code__.is_some() {
                                return Err(serde::de::Error::duplicate_field("exitCode"));
                            }
                            exit_code__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::Stdout => {
                            if stdout__.is_some() {
                                return Err(serde::de::Error::duplicate_field("stdout"));
                            }
                            stdout__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Stderr => {
                            if stderr__.is_some() {
                                return Err(serde::de::Error::duplicate_field("stderr"));
                            }
                            stderr__ = Some(map_.next_value()?);
                        }
                        GeneratedField::TmpfilePath => {
                            if tmpfile_path__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tmpfilePath"));
                            }
                            tmpfile_path__ = map_.next_value()?;
                        }
                        GeneratedField::TmpfileContents => {
                            if tmpfile_contents__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tmpfileContents"));
                            }
                            tmpfile_contents__ = map_.next_value()?;
                        }
                        GeneratedField::ExitResult => {
                            if exit_result__.is_some() {
                                return Err(serde::de::Error::duplicate_field("exitResult"));
                            }
                            exit_result__ = Some(map_.next_value::<ExitResult>()? as i32);
                        }
                        GeneratedField::ParserError => {
                            if parser_error__.is_some() {
                                return Err(serde::de::Error::duplicate_field("parserError"));
                            }
                            parser_error__ = map_.next_value()?;
                        }
                        GeneratedField::IssuesCount => {
                            if issues_count__.is_some() {
                                return Err(serde::de::Error::duplicate_field("issuesCount"));
                            }
                            issues_count__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::RewritesCount => {
                            if rewrites_count__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rewritesCount"));
                            }
                            rewrites_count__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(Invocation {
                    workspace_id: workspace_id__.unwrap_or_default(),
                    project_id: project_id__.unwrap_or_default(),
                    reference: reference__.unwrap_or_default(),
                    build_id: build_id__.unwrap_or_default(),
                    build_timestamp: build_timestamp__,
                    commit_sha: commit_sha__.unwrap_or_default(),
                    id: id__.unwrap_or_default(),
                    qlty_cli_version: qlty_cli_version__.unwrap_or_default(),
                    plugin_name: plugin_name__.unwrap_or_default(),
                    driver_name: driver_name__.unwrap_or_default(),
                    prefix: prefix__.unwrap_or_default(),
                    plugin_version: plugin_version__.unwrap_or_default(),
                    verb: verb__.unwrap_or_default(),
                    targets_count: targets_count__.unwrap_or_default(),
                    target_paths: target_paths__.unwrap_or_default(),
                    config_paths: config_paths__.unwrap_or_default(),
                    script: script__.unwrap_or_default(),
                    cwd: cwd__.unwrap_or_default(),
                    env: env__.unwrap_or_default(),
                    started_at: started_at__,
                    duration_secs: duration_secs__.unwrap_or_default(),
                    exit_code: exit_code__,
                    stdout: stdout__.unwrap_or_default(),
                    stderr: stderr__.unwrap_or_default(),
                    tmpfile_path: tmpfile_path__,
                    tmpfile_contents: tmpfile_contents__,
                    exit_result: exit_result__.unwrap_or_default(),
                    parser_error: parser_error__,
                    issues_count: issues_count__.unwrap_or_default(),
                    rewrites_count: rewrites_count__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("qlty.analysis.v1.Invocation", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Issue {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.workspace_id.is_empty() {
            len += 1;
        }
        if !self.project_id.is_empty() {
            len += 1;
        }
        if !self.id.is_empty() {
            len += 1;
        }
        if !self.reference.is_empty() {
            len += 1;
        }
        if !self.build_id.is_empty() {
            len += 1;
        }
        if !self.commit_sha.is_empty() {
            len += 1;
        }
        if self.pull_request_number.is_some() {
            len += 1;
        }
        if self.tracked_branch_id.is_some() {
            len += 1;
        }
        if self.analyzed_at.is_some() {
            len += 1;
        }
        if !self.tool.is_empty() {
            len += 1;
        }
        if !self.driver.is_empty() {
            len += 1;
        }
        if !self.rule_key.is_empty() {
            len += 1;
        }
        if !self.message.is_empty() {
            len += 1;
        }
        if self.level != 0 {
            len += 1;
        }
        if self.language != 0 {
            len += 1;
        }
        if !self.fingerprint.is_empty() {
            len += 1;
        }
        if self.category != 0 {
            len += 1;
        }
        if !self.snippet.is_empty() {
            len += 1;
        }
        if !self.snippet_with_context.is_empty() {
            len += 1;
        }
        if !self.replacement.is_empty() {
            len += 1;
        }
        if !self.documentation_url.is_empty() {
            len += 1;
        }
        if self.effort_minutes != 0 {
            len += 1;
        }
        if self.value != 0 {
            len += 1;
        }
        if self.value_delta != 0 {
            len += 1;
        }
        if !self.source_checksum.is_empty() {
            len += 1;
        }
        if self.source_checksum_version != 0 {
            len += 1;
        }
        if !self.author.is_empty() {
            len += 1;
        }
        if self.author_time.is_some() {
            len += 1;
        }
        if !self.tags.is_empty() {
            len += 1;
        }
        if self.location.is_some() {
            len += 1;
        }
        if !self.other_locations.is_empty() {
            len += 1;
        }
        if !self.suggestions.is_empty() {
            len += 1;
        }
        if self.properties.is_some() {
            len += 1;
        }
        if !self.partial_fingerprints.is_empty() {
            len += 1;
        }
        if self.mode != 0 {
            len += 1;
        }
        if self.on_added_line {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("qlty.analysis.v1.Issue", len)?;
        if !self.workspace_id.is_empty() {
            struct_ser.serialize_field("workspaceId", &self.workspace_id)?;
        }
        if !self.project_id.is_empty() {
            struct_ser.serialize_field("projectId", &self.project_id)?;
        }
        if !self.id.is_empty() {
            struct_ser.serialize_field("id", &self.id)?;
        }
        if !self.reference.is_empty() {
            struct_ser.serialize_field("reference", &self.reference)?;
        }
        if !self.build_id.is_empty() {
            struct_ser.serialize_field("buildId", &self.build_id)?;
        }
        if !self.commit_sha.is_empty() {
            struct_ser.serialize_field("commitSha", &self.commit_sha)?;
        }
        if let Some(v) = self.pull_request_number.as_ref() {
            struct_ser.serialize_field("pullRequestNumber", v)?;
        }
        if let Some(v) = self.tracked_branch_id.as_ref() {
            struct_ser.serialize_field("trackedBranchId", v)?;
        }
        if let Some(v) = self.analyzed_at.as_ref() {
            struct_ser.serialize_field("analyzedAt", v)?;
        }
        if !self.tool.is_empty() {
            struct_ser.serialize_field("tool", &self.tool)?;
        }
        if !self.driver.is_empty() {
            struct_ser.serialize_field("driver", &self.driver)?;
        }
        if !self.rule_key.is_empty() {
            struct_ser.serialize_field("ruleKey", &self.rule_key)?;
        }
        if !self.message.is_empty() {
            struct_ser.serialize_field("message", &self.message)?;
        }
        if self.level != 0 {
            let v = Level::try_from(self.level)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.level)))?;
            struct_ser.serialize_field("level", &v)?;
        }
        if self.language != 0 {
            let v = Language::try_from(self.language)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.language)))?;
            struct_ser.serialize_field("language", &v)?;
        }
        if !self.fingerprint.is_empty() {
            struct_ser.serialize_field("fingerprint", &self.fingerprint)?;
        }
        if self.category != 0 {
            let v = Category::try_from(self.category)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.category)))?;
            struct_ser.serialize_field("category", &v)?;
        }
        if !self.snippet.is_empty() {
            struct_ser.serialize_field("snippet", &self.snippet)?;
        }
        if !self.snippet_with_context.is_empty() {
            struct_ser.serialize_field("snippetWithContext", &self.snippet_with_context)?;
        }
        if !self.replacement.is_empty() {
            struct_ser.serialize_field("replacement", &self.replacement)?;
        }
        if !self.documentation_url.is_empty() {
            struct_ser.serialize_field("documentationUrl", &self.documentation_url)?;
        }
        if self.effort_minutes != 0 {
            struct_ser.serialize_field("effortMinutes", &self.effort_minutes)?;
        }
        if self.value != 0 {
            struct_ser.serialize_field("value", &self.value)?;
        }
        if self.value_delta != 0 {
            struct_ser.serialize_field("valueDelta", &self.value_delta)?;
        }
        if !self.source_checksum.is_empty() {
            struct_ser.serialize_field("sourceChecksum", &self.source_checksum)?;
        }
        if self.source_checksum_version != 0 {
            struct_ser.serialize_field("sourceChecksumVersion", &self.source_checksum_version)?;
        }
        if !self.author.is_empty() {
            struct_ser.serialize_field("author", &self.author)?;
        }
        if let Some(v) = self.author_time.as_ref() {
            struct_ser.serialize_field("authorTime", v)?;
        }
        if !self.tags.is_empty() {
            struct_ser.serialize_field("tags", &self.tags)?;
        }
        if let Some(v) = self.location.as_ref() {
            struct_ser.serialize_field("location", v)?;
        }
        if !self.other_locations.is_empty() {
            struct_ser.serialize_field("otherLocations", &self.other_locations)?;
        }
        if !self.suggestions.is_empty() {
            struct_ser.serialize_field("suggestions", &self.suggestions)?;
        }
        if let Some(v) = self.properties.as_ref() {
            struct_ser.serialize_field("properties", v)?;
        }
        if !self.partial_fingerprints.is_empty() {
            struct_ser.serialize_field("partialFingerprints", &self.partial_fingerprints)?;
        }
        if self.mode != 0 {
            let v = Mode::try_from(self.mode)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.mode)))?;
            struct_ser.serialize_field("mode", &v)?;
        }
        if self.on_added_line {
            struct_ser.serialize_field("onAddedLine", &self.on_added_line)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Issue {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "workspace_id",
            "workspaceId",
            "project_id",
            "projectId",
            "id",
            "reference",
            "build_id",
            "buildId",
            "commit_sha",
            "commitSha",
            "pull_request_number",
            "pullRequestNumber",
            "tracked_branch_id",
            "trackedBranchId",
            "analyzed_at",
            "analyzedAt",
            "tool",
            "driver",
            "rule_key",
            "ruleKey",
            "message",
            "level",
            "language",
            "fingerprint",
            "category",
            "snippet",
            "snippet_with_context",
            "snippetWithContext",
            "replacement",
            "documentation_url",
            "documentationUrl",
            "effort_minutes",
            "effortMinutes",
            "value",
            "value_delta",
            "valueDelta",
            "source_checksum",
            "sourceChecksum",
            "source_checksum_version",
            "sourceChecksumVersion",
            "author",
            "author_time",
            "authorTime",
            "tags",
            "location",
            "other_locations",
            "otherLocations",
            "suggestions",
            "properties",
            "partial_fingerprints",
            "partialFingerprints",
            "mode",
            "on_added_line",
            "onAddedLine",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            WorkspaceId,
            ProjectId,
            Id,
            Reference,
            BuildId,
            CommitSha,
            PullRequestNumber,
            TrackedBranchId,
            AnalyzedAt,
            Tool,
            Driver,
            RuleKey,
            Message,
            Level,
            Language,
            Fingerprint,
            Category,
            Snippet,
            SnippetWithContext,
            Replacement,
            DocumentationUrl,
            EffortMinutes,
            Value,
            ValueDelta,
            SourceChecksum,
            SourceChecksumVersion,
            Author,
            AuthorTime,
            Tags,
            Location,
            OtherLocations,
            Suggestions,
            Properties,
            PartialFingerprints,
            Mode,
            OnAddedLine,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "workspaceId" | "workspace_id" => Ok(GeneratedField::WorkspaceId),
                            "projectId" | "project_id" => Ok(GeneratedField::ProjectId),
                            "id" => Ok(GeneratedField::Id),
                            "reference" => Ok(GeneratedField::Reference),
                            "buildId" | "build_id" => Ok(GeneratedField::BuildId),
                            "commitSha" | "commit_sha" => Ok(GeneratedField::CommitSha),
                            "pullRequestNumber" | "pull_request_number" => Ok(GeneratedField::PullRequestNumber),
                            "trackedBranchId" | "tracked_branch_id" => Ok(GeneratedField::TrackedBranchId),
                            "analyzedAt" | "analyzed_at" => Ok(GeneratedField::AnalyzedAt),
                            "tool" => Ok(GeneratedField::Tool),
                            "driver" => Ok(GeneratedField::Driver),
                            "ruleKey" | "rule_key" => Ok(GeneratedField::RuleKey),
                            "message" => Ok(GeneratedField::Message),
                            "level" => Ok(GeneratedField::Level),
                            "language" => Ok(GeneratedField::Language),
                            "fingerprint" => Ok(GeneratedField::Fingerprint),
                            "category" => Ok(GeneratedField::Category),
                            "snippet" => Ok(GeneratedField::Snippet),
                            "snippetWithContext" | "snippet_with_context" => Ok(GeneratedField::SnippetWithContext),
                            "replacement" => Ok(GeneratedField::Replacement),
                            "documentationUrl" | "documentation_url" => Ok(GeneratedField::DocumentationUrl),
                            "effortMinutes" | "effort_minutes" => Ok(GeneratedField::EffortMinutes),
                            "value" => Ok(GeneratedField::Value),
                            "valueDelta" | "value_delta" => Ok(GeneratedField::ValueDelta),
                            "sourceChecksum" | "source_checksum" => Ok(GeneratedField::SourceChecksum),
                            "sourceChecksumVersion" | "source_checksum_version" => Ok(GeneratedField::SourceChecksumVersion),
                            "author" => Ok(GeneratedField::Author),
                            "authorTime" | "author_time" => Ok(GeneratedField::AuthorTime),
                            "tags" => Ok(GeneratedField::Tags),
                            "location" => Ok(GeneratedField::Location),
                            "otherLocations" | "other_locations" => Ok(GeneratedField::OtherLocations),
                            "suggestions" => Ok(GeneratedField::Suggestions),
                            "properties" => Ok(GeneratedField::Properties),
                            "partialFingerprints" | "partial_fingerprints" => Ok(GeneratedField::PartialFingerprints),
                            "mode" => Ok(GeneratedField::Mode),
                            "onAddedLine" | "on_added_line" => Ok(GeneratedField::OnAddedLine),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Issue;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct qlty.analysis.v1.Issue")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Issue, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut workspace_id__ = None;
                let mut project_id__ = None;
                let mut id__ = None;
                let mut reference__ = None;
                let mut build_id__ = None;
                let mut commit_sha__ = None;
                let mut pull_request_number__ = None;
                let mut tracked_branch_id__ = None;
                let mut analyzed_at__ = None;
                let mut tool__ = None;
                let mut driver__ = None;
                let mut rule_key__ = None;
                let mut message__ = None;
                let mut level__ = None;
                let mut language__ = None;
                let mut fingerprint__ = None;
                let mut category__ = None;
                let mut snippet__ = None;
                let mut snippet_with_context__ = None;
                let mut replacement__ = None;
                let mut documentation_url__ = None;
                let mut effort_minutes__ = None;
                let mut value__ = None;
                let mut value_delta__ = None;
                let mut source_checksum__ = None;
                let mut source_checksum_version__ = None;
                let mut author__ = None;
                let mut author_time__ = None;
                let mut tags__ = None;
                let mut location__ = None;
                let mut other_locations__ = None;
                let mut suggestions__ = None;
                let mut properties__ = None;
                let mut partial_fingerprints__ = None;
                let mut mode__ = None;
                let mut on_added_line__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::WorkspaceId => {
                            if workspace_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("workspaceId"));
                            }
                            workspace_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ProjectId => {
                            if project_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("projectId"));
                            }
                            project_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Id => {
                            if id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Reference => {
                            if reference__.is_some() {
                                return Err(serde::de::Error::duplicate_field("reference"));
                            }
                            reference__ = Some(map_.next_value()?);
                        }
                        GeneratedField::BuildId => {
                            if build_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("buildId"));
                            }
                            build_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::CommitSha => {
                            if commit_sha__.is_some() {
                                return Err(serde::de::Error::duplicate_field("commitSha"));
                            }
                            commit_sha__ = Some(map_.next_value()?);
                        }
                        GeneratedField::PullRequestNumber => {
                            if pull_request_number__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pullRequestNumber"));
                            }
                            pull_request_number__ = map_.next_value()?;
                        }
                        GeneratedField::TrackedBranchId => {
                            if tracked_branch_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("trackedBranchId"));
                            }
                            tracked_branch_id__ = map_.next_value()?;
                        }
                        GeneratedField::AnalyzedAt => {
                            if analyzed_at__.is_some() {
                                return Err(serde::de::Error::duplicate_field("analyzedAt"));
                            }
                            analyzed_at__ = map_.next_value()?;
                        }
                        GeneratedField::Tool => {
                            if tool__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tool"));
                            }
                            tool__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Driver => {
                            if driver__.is_some() {
                                return Err(serde::de::Error::duplicate_field("driver"));
                            }
                            driver__ = Some(map_.next_value()?);
                        }
                        GeneratedField::RuleKey => {
                            if rule_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ruleKey"));
                            }
                            rule_key__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Message => {
                            if message__.is_some() {
                                return Err(serde::de::Error::duplicate_field("message"));
                            }
                            message__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Level => {
                            if level__.is_some() {
                                return Err(serde::de::Error::duplicate_field("level"));
                            }
                            level__ = Some(map_.next_value::<Level>()? as i32);
                        }
                        GeneratedField::Language => {
                            if language__.is_some() {
                                return Err(serde::de::Error::duplicate_field("language"));
                            }
                            language__ = Some(map_.next_value::<Language>()? as i32);
                        }
                        GeneratedField::Fingerprint => {
                            if fingerprint__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fingerprint"));
                            }
                            fingerprint__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Category => {
                            if category__.is_some() {
                                return Err(serde::de::Error::duplicate_field("category"));
                            }
                            category__ = Some(map_.next_value::<Category>()? as i32);
                        }
                        GeneratedField::Snippet => {
                            if snippet__.is_some() {
                                return Err(serde::de::Error::duplicate_field("snippet"));
                            }
                            snippet__ = Some(map_.next_value()?);
                        }
                        GeneratedField::SnippetWithContext => {
                            if snippet_with_context__.is_some() {
                                return Err(serde::de::Error::duplicate_field("snippetWithContext"));
                            }
                            snippet_with_context__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Replacement => {
                            if replacement__.is_some() {
                                return Err(serde::de::Error::duplicate_field("replacement"));
                            }
                            replacement__ = Some(map_.next_value()?);
                        }
                        GeneratedField::DocumentationUrl => {
                            if documentation_url__.is_some() {
                                return Err(serde::de::Error::duplicate_field("documentationUrl"));
                            }
                            documentation_url__ = Some(map_.next_value()?);
                        }
                        GeneratedField::EffortMinutes => {
                            if effort_minutes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("effortMinutes"));
                            }
                            effort_minutes__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Value => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("value"));
                            }
                            value__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ValueDelta => {
                            if value_delta__.is_some() {
                                return Err(serde::de::Error::duplicate_field("valueDelta"));
                            }
                            value_delta__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::SourceChecksum => {
                            if source_checksum__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sourceChecksum"));
                            }
                            source_checksum__ = Some(map_.next_value()?);
                        }
                        GeneratedField::SourceChecksumVersion => {
                            if source_checksum_version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sourceChecksumVersion"));
                            }
                            source_checksum_version__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Author => {
                            if author__.is_some() {
                                return Err(serde::de::Error::duplicate_field("author"));
                            }
                            author__ = Some(map_.next_value()?);
                        }
                        GeneratedField::AuthorTime => {
                            if author_time__.is_some() {
                                return Err(serde::de::Error::duplicate_field("authorTime"));
                            }
                            author_time__ = map_.next_value()?;
                        }
                        GeneratedField::Tags => {
                            if tags__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tags"));
                            }
                            tags__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Location => {
                            if location__.is_some() {
                                return Err(serde::de::Error::duplicate_field("location"));
                            }
                            location__ = map_.next_value()?;
                        }
                        GeneratedField::OtherLocations => {
                            if other_locations__.is_some() {
                                return Err(serde::de::Error::duplicate_field("otherLocations"));
                            }
                            other_locations__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Suggestions => {
                            if suggestions__.is_some() {
                                return Err(serde::de::Error::duplicate_field("suggestions"));
                            }
                            suggestions__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Properties => {
                            if properties__.is_some() {
                                return Err(serde::de::Error::duplicate_field("properties"));
                            }
                            properties__ = map_.next_value()?;
                        }
                        GeneratedField::PartialFingerprints => {
                            if partial_fingerprints__.is_some() {
                                return Err(serde::de::Error::duplicate_field("partialFingerprints"));
                            }
                            partial_fingerprints__ = Some(
                                map_.next_value::<std::collections::HashMap<_, _>>()?
                            );
                        }
                        GeneratedField::Mode => {
                            if mode__.is_some() {
                                return Err(serde::de::Error::duplicate_field("mode"));
                            }
                            mode__ = Some(map_.next_value::<Mode>()? as i32);
                        }
                        GeneratedField::OnAddedLine => {
                            if on_added_line__.is_some() {
                                return Err(serde::de::Error::duplicate_field("onAddedLine"));
                            }
                            on_added_line__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(Issue {
                    workspace_id: workspace_id__.unwrap_or_default(),
                    project_id: project_id__.unwrap_or_default(),
                    id: id__.unwrap_or_default(),
                    reference: reference__.unwrap_or_default(),
                    build_id: build_id__.unwrap_or_default(),
                    commit_sha: commit_sha__.unwrap_or_default(),
                    pull_request_number: pull_request_number__,
                    tracked_branch_id: tracked_branch_id__,
                    analyzed_at: analyzed_at__,
                    tool: tool__.unwrap_or_default(),
                    driver: driver__.unwrap_or_default(),
                    rule_key: rule_key__.unwrap_or_default(),
                    message: message__.unwrap_or_default(),
                    level: level__.unwrap_or_default(),
                    language: language__.unwrap_or_default(),
                    fingerprint: fingerprint__.unwrap_or_default(),
                    category: category__.unwrap_or_default(),
                    snippet: snippet__.unwrap_or_default(),
                    snippet_with_context: snippet_with_context__.unwrap_or_default(),
                    replacement: replacement__.unwrap_or_default(),
                    documentation_url: documentation_url__.unwrap_or_default(),
                    effort_minutes: effort_minutes__.unwrap_or_default(),
                    value: value__.unwrap_or_default(),
                    value_delta: value_delta__.unwrap_or_default(),
                    source_checksum: source_checksum__.unwrap_or_default(),
                    source_checksum_version: source_checksum_version__.unwrap_or_default(),
                    author: author__.unwrap_or_default(),
                    author_time: author_time__,
                    tags: tags__.unwrap_or_default(),
                    location: location__,
                    other_locations: other_locations__.unwrap_or_default(),
                    suggestions: suggestions__.unwrap_or_default(),
                    properties: properties__,
                    partial_fingerprints: partial_fingerprints__.unwrap_or_default(),
                    mode: mode__.unwrap_or_default(),
                    on_added_line: on_added_line__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("qlty.analysis.v1.Issue", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Language {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "LANGUAGE_UNSPECIFIED",
            Self::Unknown => "LANGUAGE_UNKNOWN",
            Self::Java => "LANGUAGE_JAVA",
            Self::Javascript => "LANGUAGE_JAVASCRIPT",
            Self::Typescript => "LANGUAGE_TYPESCRIPT",
            Self::Python => "LANGUAGE_PYTHON",
            Self::Ruby => "LANGUAGE_RUBY",
            Self::Jsx => "LANGUAGE_JSX",
            Self::Tsx => "LANGUAGE_TSX",
            Self::Go => "LANGUAGE_GO",
            Self::Rust => "LANGUAGE_RUST",
            Self::Kotlin => "LANGUAGE_KOTLIN",
            Self::Php => "LANGUAGE_PHP",
            Self::CSharp => "LANGUAGE_C_SHARP",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for Language {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "LANGUAGE_UNSPECIFIED",
            "LANGUAGE_UNKNOWN",
            "LANGUAGE_JAVA",
            "LANGUAGE_JAVASCRIPT",
            "LANGUAGE_TYPESCRIPT",
            "LANGUAGE_PYTHON",
            "LANGUAGE_RUBY",
            "LANGUAGE_JSX",
            "LANGUAGE_TSX",
            "LANGUAGE_GO",
            "LANGUAGE_RUST",
            "LANGUAGE_KOTLIN",
            "LANGUAGE_PHP",
            "LANGUAGE_C_SHARP",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Language;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "LANGUAGE_UNSPECIFIED" => Ok(Language::Unspecified),
                    "LANGUAGE_UNKNOWN" => Ok(Language::Unknown),
                    "LANGUAGE_JAVA" => Ok(Language::Java),
                    "LANGUAGE_JAVASCRIPT" => Ok(Language::Javascript),
                    "LANGUAGE_TYPESCRIPT" => Ok(Language::Typescript),
                    "LANGUAGE_PYTHON" => Ok(Language::Python),
                    "LANGUAGE_RUBY" => Ok(Language::Ruby),
                    "LANGUAGE_JSX" => Ok(Language::Jsx),
                    "LANGUAGE_TSX" => Ok(Language::Tsx),
                    "LANGUAGE_GO" => Ok(Language::Go),
                    "LANGUAGE_RUST" => Ok(Language::Rust),
                    "LANGUAGE_KOTLIN" => Ok(Language::Kotlin),
                    "LANGUAGE_PHP" => Ok(Language::Php),
                    "LANGUAGE_C_SHARP" => Ok(Language::CSharp),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for Level {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "LEVEL_UNSPECIFIED",
            Self::Note => "LEVEL_NOTE",
            Self::Fmt => "LEVEL_FMT",
            Self::Low => "LEVEL_LOW",
            Self::Medium => "LEVEL_MEDIUM",
            Self::High => "LEVEL_HIGH",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for Level {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "LEVEL_UNSPECIFIED",
            "LEVEL_NOTE",
            "LEVEL_FMT",
            "LEVEL_LOW",
            "LEVEL_MEDIUM",
            "LEVEL_HIGH",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Level;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "LEVEL_UNSPECIFIED" => Ok(Level::Unspecified),
                    "LEVEL_NOTE" => Ok(Level::Note),
                    "LEVEL_FMT" => Ok(Level::Fmt),
                    "LEVEL_LOW" => Ok(Level::Low),
                    "LEVEL_MEDIUM" => Ok(Level::Medium),
                    "LEVEL_HIGH" => Ok(Level::High),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for Location {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.path.is_empty() {
            len += 1;
        }
        if self.range.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("qlty.analysis.v1.Location", len)?;
        if !self.path.is_empty() {
            struct_ser.serialize_field("path", &self.path)?;
        }
        if let Some(v) = self.range.as_ref() {
            struct_ser.serialize_field("range", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Location {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "path",
            "range",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Path,
            Range,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "path" => Ok(GeneratedField::Path),
                            "range" => Ok(GeneratedField::Range),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Location;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct qlty.analysis.v1.Location")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Location, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut path__ = None;
                let mut range__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Path => {
                            if path__.is_some() {
                                return Err(serde::de::Error::duplicate_field("path"));
                            }
                            path__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Range => {
                            if range__.is_some() {
                                return Err(serde::de::Error::duplicate_field("range"));
                            }
                            range__ = map_.next_value()?;
                        }
                    }
                }
                Ok(Location {
                    path: path__.unwrap_or_default(),
                    range: range__,
                })
            }
        }
        deserializer.deserialize_struct("qlty.analysis.v1.Location", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Message {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.workspace_id.is_empty() {
            len += 1;
        }
        if !self.project_id.is_empty() {
            len += 1;
        }
        if !self.reference.is_empty() {
            len += 1;
        }
        if !self.build_id.is_empty() {
            len += 1;
        }
        if self.build_timestamp.is_some() {
            len += 1;
        }
        if !self.commit_sha.is_empty() {
            len += 1;
        }
        if self.timestamp.is_some() {
            len += 1;
        }
        if !self.module.is_empty() {
            len += 1;
        }
        if !self.ty.is_empty() {
            len += 1;
        }
        if !self.message.is_empty() {
            len += 1;
        }
        if !self.details.is_empty() {
            len += 1;
        }
        if self.level != 0 {
            len += 1;
        }
        if !self.tags.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("qlty.analysis.v1.Message", len)?;
        if !self.workspace_id.is_empty() {
            struct_ser.serialize_field("workspaceId", &self.workspace_id)?;
        }
        if !self.project_id.is_empty() {
            struct_ser.serialize_field("projectId", &self.project_id)?;
        }
        if !self.reference.is_empty() {
            struct_ser.serialize_field("reference", &self.reference)?;
        }
        if !self.build_id.is_empty() {
            struct_ser.serialize_field("buildId", &self.build_id)?;
        }
        if let Some(v) = self.build_timestamp.as_ref() {
            struct_ser.serialize_field("buildTimestamp", v)?;
        }
        if !self.commit_sha.is_empty() {
            struct_ser.serialize_field("commitSha", &self.commit_sha)?;
        }
        if let Some(v) = self.timestamp.as_ref() {
            struct_ser.serialize_field("timestamp", v)?;
        }
        if !self.module.is_empty() {
            struct_ser.serialize_field("module", &self.module)?;
        }
        if !self.ty.is_empty() {
            struct_ser.serialize_field("ty", &self.ty)?;
        }
        if !self.message.is_empty() {
            struct_ser.serialize_field("message", &self.message)?;
        }
        if !self.details.is_empty() {
            struct_ser.serialize_field("details", &self.details)?;
        }
        if self.level != 0 {
            let v = MessageLevel::try_from(self.level)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.level)))?;
            struct_ser.serialize_field("level", &v)?;
        }
        if !self.tags.is_empty() {
            struct_ser.serialize_field("tags", &self.tags)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Message {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "workspace_id",
            "workspaceId",
            "project_id",
            "projectId",
            "reference",
            "build_id",
            "buildId",
            "build_timestamp",
            "buildTimestamp",
            "commit_sha",
            "commitSha",
            "timestamp",
            "module",
            "ty",
            "message",
            "details",
            "level",
            "tags",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            WorkspaceId,
            ProjectId,
            Reference,
            BuildId,
            BuildTimestamp,
            CommitSha,
            Timestamp,
            Module,
            Ty,
            Message,
            Details,
            Level,
            Tags,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "workspaceId" | "workspace_id" => Ok(GeneratedField::WorkspaceId),
                            "projectId" | "project_id" => Ok(GeneratedField::ProjectId),
                            "reference" => Ok(GeneratedField::Reference),
                            "buildId" | "build_id" => Ok(GeneratedField::BuildId),
                            "buildTimestamp" | "build_timestamp" => Ok(GeneratedField::BuildTimestamp),
                            "commitSha" | "commit_sha" => Ok(GeneratedField::CommitSha),
                            "timestamp" => Ok(GeneratedField::Timestamp),
                            "module" => Ok(GeneratedField::Module),
                            "ty" => Ok(GeneratedField::Ty),
                            "message" => Ok(GeneratedField::Message),
                            "details" => Ok(GeneratedField::Details),
                            "level" => Ok(GeneratedField::Level),
                            "tags" => Ok(GeneratedField::Tags),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Message;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct qlty.analysis.v1.Message")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Message, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut workspace_id__ = None;
                let mut project_id__ = None;
                let mut reference__ = None;
                let mut build_id__ = None;
                let mut build_timestamp__ = None;
                let mut commit_sha__ = None;
                let mut timestamp__ = None;
                let mut module__ = None;
                let mut ty__ = None;
                let mut message__ = None;
                let mut details__ = None;
                let mut level__ = None;
                let mut tags__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::WorkspaceId => {
                            if workspace_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("workspaceId"));
                            }
                            workspace_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ProjectId => {
                            if project_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("projectId"));
                            }
                            project_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Reference => {
                            if reference__.is_some() {
                                return Err(serde::de::Error::duplicate_field("reference"));
                            }
                            reference__ = Some(map_.next_value()?);
                        }
                        GeneratedField::BuildId => {
                            if build_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("buildId"));
                            }
                            build_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::BuildTimestamp => {
                            if build_timestamp__.is_some() {
                                return Err(serde::de::Error::duplicate_field("buildTimestamp"));
                            }
                            build_timestamp__ = map_.next_value()?;
                        }
                        GeneratedField::CommitSha => {
                            if commit_sha__.is_some() {
                                return Err(serde::de::Error::duplicate_field("commitSha"));
                            }
                            commit_sha__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Timestamp => {
                            if timestamp__.is_some() {
                                return Err(serde::de::Error::duplicate_field("timestamp"));
                            }
                            timestamp__ = map_.next_value()?;
                        }
                        GeneratedField::Module => {
                            if module__.is_some() {
                                return Err(serde::de::Error::duplicate_field("module"));
                            }
                            module__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Ty => {
                            if ty__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ty"));
                            }
                            ty__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Message => {
                            if message__.is_some() {
                                return Err(serde::de::Error::duplicate_field("message"));
                            }
                            message__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Details => {
                            if details__.is_some() {
                                return Err(serde::de::Error::duplicate_field("details"));
                            }
                            details__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Level => {
                            if level__.is_some() {
                                return Err(serde::de::Error::duplicate_field("level"));
                            }
                            level__ = Some(map_.next_value::<MessageLevel>()? as i32);
                        }
                        GeneratedField::Tags => {
                            if tags__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tags"));
                            }
                            tags__ = Some(
                                map_.next_value::<std::collections::HashMap<_, _>>()?
                            );
                        }
                    }
                }
                Ok(Message {
                    workspace_id: workspace_id__.unwrap_or_default(),
                    project_id: project_id__.unwrap_or_default(),
                    reference: reference__.unwrap_or_default(),
                    build_id: build_id__.unwrap_or_default(),
                    build_timestamp: build_timestamp__,
                    commit_sha: commit_sha__.unwrap_or_default(),
                    timestamp: timestamp__,
                    module: module__.unwrap_or_default(),
                    ty: ty__.unwrap_or_default(),
                    message: message__.unwrap_or_default(),
                    details: details__.unwrap_or_default(),
                    level: level__.unwrap_or_default(),
                    tags: tags__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("qlty.analysis.v1.Message", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MessageLevel {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "MESSAGE_LEVEL_UNSPECIFIED",
            Self::Debug => "MESSAGE_LEVEL_DEBUG",
            Self::Info => "MESSAGE_LEVEL_INFO",
            Self::Warning => "MESSAGE_LEVEL_WARNING",
            Self::Error => "MESSAGE_LEVEL_ERROR",
            Self::Fatal => "MESSAGE_LEVEL_FATAL",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for MessageLevel {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "MESSAGE_LEVEL_UNSPECIFIED",
            "MESSAGE_LEVEL_DEBUG",
            "MESSAGE_LEVEL_INFO",
            "MESSAGE_LEVEL_WARNING",
            "MESSAGE_LEVEL_ERROR",
            "MESSAGE_LEVEL_FATAL",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MessageLevel;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "MESSAGE_LEVEL_UNSPECIFIED" => Ok(MessageLevel::Unspecified),
                    "MESSAGE_LEVEL_DEBUG" => Ok(MessageLevel::Debug),
                    "MESSAGE_LEVEL_INFO" => Ok(MessageLevel::Info),
                    "MESSAGE_LEVEL_WARNING" => Ok(MessageLevel::Warning),
                    "MESSAGE_LEVEL_ERROR" => Ok(MessageLevel::Error),
                    "MESSAGE_LEVEL_FATAL" => Ok(MessageLevel::Fatal),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for Metadata {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.workspace_id.is_empty() {
            len += 1;
        }
        if !self.project_id.is_empty() {
            len += 1;
        }
        if !self.build_id.is_empty() {
            len += 1;
        }
        if !self.reference.is_empty() {
            len += 1;
        }
        if self.pull_request_number.is_some() {
            len += 1;
        }
        if self.tracked_branch_id.is_some() {
            len += 1;
        }
        if !self.revision_oid.is_empty() {
            len += 1;
        }
        if self.result != 0 {
            len += 1;
        }
        if !self.branch.is_empty() {
            len += 1;
        }
        if self.backfill {
            len += 1;
        }
        if !self.root_directory.is_empty() {
            len += 1;
        }
        if !self.repository_clone_url.is_empty() {
            len += 1;
        }
        if self.files_analyzed.is_some() {
            len += 1;
        }
        if self.start_time.is_some() {
            len += 1;
        }
        if self.finish_time.is_some() {
            len += 1;
        }
        if !self.commit_message.is_empty() {
            len += 1;
        }
        if self.committed_at.is_some() {
            len += 1;
        }
        if !self.committer_email.is_empty() {
            len += 1;
        }
        if !self.committer_name.is_empty() {
            len += 1;
        }
        if !self.author_email.is_empty() {
            len += 1;
        }
        if !self.author_name.is_empty() {
            len += 1;
        }
        if self.authored_at.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("qlty.analysis.v1.Metadata", len)?;
        if !self.workspace_id.is_empty() {
            struct_ser.serialize_field("workspaceId", &self.workspace_id)?;
        }
        if !self.project_id.is_empty() {
            struct_ser.serialize_field("projectId", &self.project_id)?;
        }
        if !self.build_id.is_empty() {
            struct_ser.serialize_field("buildId", &self.build_id)?;
        }
        if !self.reference.is_empty() {
            struct_ser.serialize_field("reference", &self.reference)?;
        }
        if let Some(v) = self.pull_request_number.as_ref() {
            struct_ser.serialize_field("pullRequestNumber", v)?;
        }
        if let Some(v) = self.tracked_branch_id.as_ref() {
            struct_ser.serialize_field("trackedBranchId", v)?;
        }
        if !self.revision_oid.is_empty() {
            struct_ser.serialize_field("revisionOid", &self.revision_oid)?;
        }
        if self.result != 0 {
            let v = AnalysisResult::try_from(self.result)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.result)))?;
            struct_ser.serialize_field("result", &v)?;
        }
        if !self.branch.is_empty() {
            struct_ser.serialize_field("branch", &self.branch)?;
        }
        if self.backfill {
            struct_ser.serialize_field("backfill", &self.backfill)?;
        }
        if !self.root_directory.is_empty() {
            struct_ser.serialize_field("rootDirectory", &self.root_directory)?;
        }
        if !self.repository_clone_url.is_empty() {
            struct_ser.serialize_field("repositoryCloneUrl", &self.repository_clone_url)?;
        }
        if let Some(v) = self.files_analyzed.as_ref() {
            struct_ser.serialize_field("filesAnalyzed", v)?;
        }
        if let Some(v) = self.start_time.as_ref() {
            struct_ser.serialize_field("startTime", v)?;
        }
        if let Some(v) = self.finish_time.as_ref() {
            struct_ser.serialize_field("finishTime", v)?;
        }
        if !self.commit_message.is_empty() {
            struct_ser.serialize_field("commitMessage", &self.commit_message)?;
        }
        if let Some(v) = self.committed_at.as_ref() {
            struct_ser.serialize_field("committedAt", v)?;
        }
        if !self.committer_email.is_empty() {
            struct_ser.serialize_field("committerEmail", &self.committer_email)?;
        }
        if !self.committer_name.is_empty() {
            struct_ser.serialize_field("committerName", &self.committer_name)?;
        }
        if !self.author_email.is_empty() {
            struct_ser.serialize_field("authorEmail", &self.author_email)?;
        }
        if !self.author_name.is_empty() {
            struct_ser.serialize_field("authorName", &self.author_name)?;
        }
        if let Some(v) = self.authored_at.as_ref() {
            struct_ser.serialize_field("authoredAt", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Metadata {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "workspace_id",
            "workspaceId",
            "project_id",
            "projectId",
            "build_id",
            "buildId",
            "reference",
            "pull_request_number",
            "pullRequestNumber",
            "tracked_branch_id",
            "trackedBranchId",
            "revision_oid",
            "revisionOid",
            "result",
            "branch",
            "backfill",
            "root_directory",
            "rootDirectory",
            "repository_clone_url",
            "repositoryCloneUrl",
            "files_analyzed",
            "filesAnalyzed",
            "start_time",
            "startTime",
            "finish_time",
            "finishTime",
            "commit_message",
            "commitMessage",
            "committed_at",
            "committedAt",
            "committer_email",
            "committerEmail",
            "committer_name",
            "committerName",
            "author_email",
            "authorEmail",
            "author_name",
            "authorName",
            "authored_at",
            "authoredAt",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            WorkspaceId,
            ProjectId,
            BuildId,
            Reference,
            PullRequestNumber,
            TrackedBranchId,
            RevisionOid,
            Result,
            Branch,
            Backfill,
            RootDirectory,
            RepositoryCloneUrl,
            FilesAnalyzed,
            StartTime,
            FinishTime,
            CommitMessage,
            CommittedAt,
            CommitterEmail,
            CommitterName,
            AuthorEmail,
            AuthorName,
            AuthoredAt,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "workspaceId" | "workspace_id" => Ok(GeneratedField::WorkspaceId),
                            "projectId" | "project_id" => Ok(GeneratedField::ProjectId),
                            "buildId" | "build_id" => Ok(GeneratedField::BuildId),
                            "reference" => Ok(GeneratedField::Reference),
                            "pullRequestNumber" | "pull_request_number" => Ok(GeneratedField::PullRequestNumber),
                            "trackedBranchId" | "tracked_branch_id" => Ok(GeneratedField::TrackedBranchId),
                            "revisionOid" | "revision_oid" => Ok(GeneratedField::RevisionOid),
                            "result" => Ok(GeneratedField::Result),
                            "branch" => Ok(GeneratedField::Branch),
                            "backfill" => Ok(GeneratedField::Backfill),
                            "rootDirectory" | "root_directory" => Ok(GeneratedField::RootDirectory),
                            "repositoryCloneUrl" | "repository_clone_url" => Ok(GeneratedField::RepositoryCloneUrl),
                            "filesAnalyzed" | "files_analyzed" => Ok(GeneratedField::FilesAnalyzed),
                            "startTime" | "start_time" => Ok(GeneratedField::StartTime),
                            "finishTime" | "finish_time" => Ok(GeneratedField::FinishTime),
                            "commitMessage" | "commit_message" => Ok(GeneratedField::CommitMessage),
                            "committedAt" | "committed_at" => Ok(GeneratedField::CommittedAt),
                            "committerEmail" | "committer_email" => Ok(GeneratedField::CommitterEmail),
                            "committerName" | "committer_name" => Ok(GeneratedField::CommitterName),
                            "authorEmail" | "author_email" => Ok(GeneratedField::AuthorEmail),
                            "authorName" | "author_name" => Ok(GeneratedField::AuthorName),
                            "authoredAt" | "authored_at" => Ok(GeneratedField::AuthoredAt),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Metadata;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct qlty.analysis.v1.Metadata")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Metadata, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut workspace_id__ = None;
                let mut project_id__ = None;
                let mut build_id__ = None;
                let mut reference__ = None;
                let mut pull_request_number__ = None;
                let mut tracked_branch_id__ = None;
                let mut revision_oid__ = None;
                let mut result__ = None;
                let mut branch__ = None;
                let mut backfill__ = None;
                let mut root_directory__ = None;
                let mut repository_clone_url__ = None;
                let mut files_analyzed__ = None;
                let mut start_time__ = None;
                let mut finish_time__ = None;
                let mut commit_message__ = None;
                let mut committed_at__ = None;
                let mut committer_email__ = None;
                let mut committer_name__ = None;
                let mut author_email__ = None;
                let mut author_name__ = None;
                let mut authored_at__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::WorkspaceId => {
                            if workspace_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("workspaceId"));
                            }
                            workspace_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ProjectId => {
                            if project_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("projectId"));
                            }
                            project_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::BuildId => {
                            if build_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("buildId"));
                            }
                            build_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Reference => {
                            if reference__.is_some() {
                                return Err(serde::de::Error::duplicate_field("reference"));
                            }
                            reference__ = Some(map_.next_value()?);
                        }
                        GeneratedField::PullRequestNumber => {
                            if pull_request_number__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pullRequestNumber"));
                            }
                            pull_request_number__ = map_.next_value()?;
                        }
                        GeneratedField::TrackedBranchId => {
                            if tracked_branch_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("trackedBranchId"));
                            }
                            tracked_branch_id__ = map_.next_value()?;
                        }
                        GeneratedField::RevisionOid => {
                            if revision_oid__.is_some() {
                                return Err(serde::de::Error::duplicate_field("revisionOid"));
                            }
                            revision_oid__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Result => {
                            if result__.is_some() {
                                return Err(serde::de::Error::duplicate_field("result"));
                            }
                            result__ = Some(map_.next_value::<AnalysisResult>()? as i32);
                        }
                        GeneratedField::Branch => {
                            if branch__.is_some() {
                                return Err(serde::de::Error::duplicate_field("branch"));
                            }
                            branch__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Backfill => {
                            if backfill__.is_some() {
                                return Err(serde::de::Error::duplicate_field("backfill"));
                            }
                            backfill__ = Some(map_.next_value()?);
                        }
                        GeneratedField::RootDirectory => {
                            if root_directory__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rootDirectory"));
                            }
                            root_directory__ = Some(map_.next_value()?);
                        }
                        GeneratedField::RepositoryCloneUrl => {
                            if repository_clone_url__.is_some() {
                                return Err(serde::de::Error::duplicate_field("repositoryCloneUrl"));
                            }
                            repository_clone_url__ = Some(map_.next_value()?);
                        }
                        GeneratedField::FilesAnalyzed => {
                            if files_analyzed__.is_some() {
                                return Err(serde::de::Error::duplicate_field("filesAnalyzed"));
                            }
                            files_analyzed__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::StartTime => {
                            if start_time__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startTime"));
                            }
                            start_time__ = map_.next_value()?;
                        }
                        GeneratedField::FinishTime => {
                            if finish_time__.is_some() {
                                return Err(serde::de::Error::duplicate_field("finishTime"));
                            }
                            finish_time__ = map_.next_value()?;
                        }
                        GeneratedField::CommitMessage => {
                            if commit_message__.is_some() {
                                return Err(serde::de::Error::duplicate_field("commitMessage"));
                            }
                            commit_message__ = Some(map_.next_value()?);
                        }
                        GeneratedField::CommittedAt => {
                            if committed_at__.is_some() {
                                return Err(serde::de::Error::duplicate_field("committedAt"));
                            }
                            committed_at__ = map_.next_value()?;
                        }
                        GeneratedField::CommitterEmail => {
                            if committer_email__.is_some() {
                                return Err(serde::de::Error::duplicate_field("committerEmail"));
                            }
                            committer_email__ = Some(map_.next_value()?);
                        }
                        GeneratedField::CommitterName => {
                            if committer_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("committerName"));
                            }
                            committer_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::AuthorEmail => {
                            if author_email__.is_some() {
                                return Err(serde::de::Error::duplicate_field("authorEmail"));
                            }
                            author_email__ = Some(map_.next_value()?);
                        }
                        GeneratedField::AuthorName => {
                            if author_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("authorName"));
                            }
                            author_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::AuthoredAt => {
                            if authored_at__.is_some() {
                                return Err(serde::de::Error::duplicate_field("authoredAt"));
                            }
                            authored_at__ = map_.next_value()?;
                        }
                    }
                }
                Ok(Metadata {
                    workspace_id: workspace_id__.unwrap_or_default(),
                    project_id: project_id__.unwrap_or_default(),
                    build_id: build_id__.unwrap_or_default(),
                    reference: reference__.unwrap_or_default(),
                    pull_request_number: pull_request_number__,
                    tracked_branch_id: tracked_branch_id__,
                    revision_oid: revision_oid__.unwrap_or_default(),
                    result: result__.unwrap_or_default(),
                    branch: branch__.unwrap_or_default(),
                    backfill: backfill__.unwrap_or_default(),
                    root_directory: root_directory__.unwrap_or_default(),
                    repository_clone_url: repository_clone_url__.unwrap_or_default(),
                    files_analyzed: files_analyzed__,
                    start_time: start_time__,
                    finish_time: finish_time__,
                    commit_message: commit_message__.unwrap_or_default(),
                    committed_at: committed_at__,
                    committer_email: committer_email__.unwrap_or_default(),
                    committer_name: committer_name__.unwrap_or_default(),
                    author_email: author_email__.unwrap_or_default(),
                    author_name: author_name__.unwrap_or_default(),
                    authored_at: authored_at__,
                })
            }
        }
        deserializer.deserialize_struct("qlty.analysis.v1.Metadata", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Mode {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "MODE_UNSPECIFIED",
            Self::Block => "MODE_BLOCK",
            Self::Comment => "MODE_COMMENT",
            Self::Monitor => "MODE_MONITOR",
            Self::Disabled => "MODE_DISABLED",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for Mode {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "MODE_UNSPECIFIED",
            "MODE_BLOCK",
            "MODE_COMMENT",
            "MODE_MONITOR",
            "MODE_DISABLED",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Mode;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "MODE_UNSPECIFIED" => Ok(Mode::Unspecified),
                    "MODE_BLOCK" => Ok(Mode::Block),
                    "MODE_COMMENT" => Ok(Mode::Comment),
                    "MODE_MONITOR" => Ok(Mode::Monitor),
                    "MODE_DISABLED" => Ok(Mode::Disabled),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for Range {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.start_line != 0 {
            len += 1;
        }
        if self.start_column != 0 {
            len += 1;
        }
        if self.end_line != 0 {
            len += 1;
        }
        if self.end_column != 0 {
            len += 1;
        }
        if self.start_byte.is_some() {
            len += 1;
        }
        if self.end_byte.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("qlty.analysis.v1.Range", len)?;
        if self.start_line != 0 {
            struct_ser.serialize_field("startLine", &self.start_line)?;
        }
        if self.start_column != 0 {
            struct_ser.serialize_field("startColumn", &self.start_column)?;
        }
        if self.end_line != 0 {
            struct_ser.serialize_field("endLine", &self.end_line)?;
        }
        if self.end_column != 0 {
            struct_ser.serialize_field("endColumn", &self.end_column)?;
        }
        if let Some(v) = self.start_byte.as_ref() {
            struct_ser.serialize_field("startByte", v)?;
        }
        if let Some(v) = self.end_byte.as_ref() {
            struct_ser.serialize_field("endByte", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Range {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "start_line",
            "startLine",
            "start_column",
            "startColumn",
            "end_line",
            "endLine",
            "end_column",
            "endColumn",
            "start_byte",
            "startByte",
            "end_byte",
            "endByte",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            StartLine,
            StartColumn,
            EndLine,
            EndColumn,
            StartByte,
            EndByte,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "startLine" | "start_line" => Ok(GeneratedField::StartLine),
                            "startColumn" | "start_column" => Ok(GeneratedField::StartColumn),
                            "endLine" | "end_line" => Ok(GeneratedField::EndLine),
                            "endColumn" | "end_column" => Ok(GeneratedField::EndColumn),
                            "startByte" | "start_byte" => Ok(GeneratedField::StartByte),
                            "endByte" | "end_byte" => Ok(GeneratedField::EndByte),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Range;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct qlty.analysis.v1.Range")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Range, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut start_line__ = None;
                let mut start_column__ = None;
                let mut end_line__ = None;
                let mut end_column__ = None;
                let mut start_byte__ = None;
                let mut end_byte__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::StartLine => {
                            if start_line__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startLine"));
                            }
                            start_line__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::StartColumn => {
                            if start_column__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startColumn"));
                            }
                            start_column__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::EndLine => {
                            if end_line__.is_some() {
                                return Err(serde::de::Error::duplicate_field("endLine"));
                            }
                            end_line__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::EndColumn => {
                            if end_column__.is_some() {
                                return Err(serde::de::Error::duplicate_field("endColumn"));
                            }
                            end_column__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::StartByte => {
                            if start_byte__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startByte"));
                            }
                            start_byte__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::EndByte => {
                            if end_byte__.is_some() {
                                return Err(serde::de::Error::duplicate_field("endByte"));
                            }
                            end_byte__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                    }
                }
                Ok(Range {
                    start_line: start_line__.unwrap_or_default(),
                    start_column: start_column__.unwrap_or_default(),
                    end_line: end_line__.unwrap_or_default(),
                    end_column: end_column__.unwrap_or_default(),
                    start_byte: start_byte__,
                    end_byte: end_byte__,
                })
            }
        }
        deserializer.deserialize_struct("qlty.analysis.v1.Range", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Replacement {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.data.is_empty() {
            len += 1;
        }
        if self.location.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("qlty.analysis.v1.Replacement", len)?;
        if !self.data.is_empty() {
            struct_ser.serialize_field("data", &self.data)?;
        }
        if let Some(v) = self.location.as_ref() {
            struct_ser.serialize_field("location", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Replacement {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "data",
            "location",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Data,
            Location,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "data" => Ok(GeneratedField::Data),
                            "location" => Ok(GeneratedField::Location),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Replacement;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct qlty.analysis.v1.Replacement")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Replacement, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut data__ = None;
                let mut location__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Data => {
                            if data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("data"));
                            }
                            data__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Location => {
                            if location__.is_some() {
                                return Err(serde::de::Error::duplicate_field("location"));
                            }
                            location__ = map_.next_value()?;
                        }
                    }
                }
                Ok(Replacement {
                    data: data__.unwrap_or_default(),
                    location: location__,
                })
            }
        }
        deserializer.deserialize_struct("qlty.analysis.v1.Replacement", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Stats {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.workspace_id.is_empty() {
            len += 1;
        }
        if !self.project_id.is_empty() {
            len += 1;
        }
        if !self.reference.is_empty() {
            len += 1;
        }
        if !self.build_id.is_empty() {
            len += 1;
        }
        if !self.commit_sha.is_empty() {
            len += 1;
        }
        if self.pull_request_number.is_some() {
            len += 1;
        }
        if self.tracked_branch_id.is_some() {
            len += 1;
        }
        if self.analyzed_at.is_some() {
            len += 1;
        }
        if !self.name.is_empty() {
            len += 1;
        }
        if !self.fully_qualified_name.is_empty() {
            len += 1;
        }
        if !self.path.is_empty() {
            len += 1;
        }
        if self.kind != 0 {
            len += 1;
        }
        if self.language != 0 {
            len += 1;
        }
        if self.files.is_some() {
            len += 1;
        }
        if self.classes.is_some() {
            len += 1;
        }
        if self.functions.is_some() {
            len += 1;
        }
        if self.fields.is_some() {
            len += 1;
        }
        if self.lines.is_some() {
            len += 1;
        }
        if self.code_lines.is_some() {
            len += 1;
        }
        if self.comment_lines.is_some() {
            len += 1;
        }
        if self.blank_lines.is_some() {
            len += 1;
        }
        if self.complexity.is_some() {
            len += 1;
        }
        if self.cyclomatic.is_some() {
            len += 1;
        }
        if self.lcom4.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("qlty.analysis.v1.Stats", len)?;
        if !self.workspace_id.is_empty() {
            struct_ser.serialize_field("workspaceId", &self.workspace_id)?;
        }
        if !self.project_id.is_empty() {
            struct_ser.serialize_field("projectId", &self.project_id)?;
        }
        if !self.reference.is_empty() {
            struct_ser.serialize_field("reference", &self.reference)?;
        }
        if !self.build_id.is_empty() {
            struct_ser.serialize_field("buildId", &self.build_id)?;
        }
        if !self.commit_sha.is_empty() {
            struct_ser.serialize_field("commitSha", &self.commit_sha)?;
        }
        if let Some(v) = self.pull_request_number.as_ref() {
            struct_ser.serialize_field("pullRequestNumber", v)?;
        }
        if let Some(v) = self.tracked_branch_id.as_ref() {
            struct_ser.serialize_field("trackedBranchId", v)?;
        }
        if let Some(v) = self.analyzed_at.as_ref() {
            struct_ser.serialize_field("analyzedAt", v)?;
        }
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if !self.fully_qualified_name.is_empty() {
            struct_ser.serialize_field("fullyQualifiedName", &self.fully_qualified_name)?;
        }
        if !self.path.is_empty() {
            struct_ser.serialize_field("path", &self.path)?;
        }
        if self.kind != 0 {
            let v = ComponentType::try_from(self.kind)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.kind)))?;
            struct_ser.serialize_field("kind", &v)?;
        }
        if self.language != 0 {
            let v = Language::try_from(self.language)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.language)))?;
            struct_ser.serialize_field("language", &v)?;
        }
        if let Some(v) = self.files.as_ref() {
            struct_ser.serialize_field("files", v)?;
        }
        if let Some(v) = self.classes.as_ref() {
            struct_ser.serialize_field("classes", v)?;
        }
        if let Some(v) = self.functions.as_ref() {
            struct_ser.serialize_field("functions", v)?;
        }
        if let Some(v) = self.fields.as_ref() {
            struct_ser.serialize_field("fields", v)?;
        }
        if let Some(v) = self.lines.as_ref() {
            struct_ser.serialize_field("lines", v)?;
        }
        if let Some(v) = self.code_lines.as_ref() {
            struct_ser.serialize_field("codeLines", v)?;
        }
        if let Some(v) = self.comment_lines.as_ref() {
            struct_ser.serialize_field("commentLines", v)?;
        }
        if let Some(v) = self.blank_lines.as_ref() {
            struct_ser.serialize_field("blankLines", v)?;
        }
        if let Some(v) = self.complexity.as_ref() {
            struct_ser.serialize_field("complexity", v)?;
        }
        if let Some(v) = self.cyclomatic.as_ref() {
            struct_ser.serialize_field("cyclomatic", v)?;
        }
        if let Some(v) = self.lcom4.as_ref() {
            struct_ser.serialize_field("lcom4", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Stats {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "workspace_id",
            "workspaceId",
            "project_id",
            "projectId",
            "reference",
            "build_id",
            "buildId",
            "commit_sha",
            "commitSha",
            "pull_request_number",
            "pullRequestNumber",
            "tracked_branch_id",
            "trackedBranchId",
            "analyzed_at",
            "analyzedAt",
            "name",
            "fully_qualified_name",
            "fullyQualifiedName",
            "path",
            "kind",
            "language",
            "files",
            "classes",
            "functions",
            "fields",
            "lines",
            "code_lines",
            "codeLines",
            "comment_lines",
            "commentLines",
            "blank_lines",
            "blankLines",
            "complexity",
            "cyclomatic",
            "lcom4",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            WorkspaceId,
            ProjectId,
            Reference,
            BuildId,
            CommitSha,
            PullRequestNumber,
            TrackedBranchId,
            AnalyzedAt,
            Name,
            FullyQualifiedName,
            Path,
            Kind,
            Language,
            Files,
            Classes,
            Functions,
            Fields,
            Lines,
            CodeLines,
            CommentLines,
            BlankLines,
            Complexity,
            Cyclomatic,
            Lcom4,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "workspaceId" | "workspace_id" => Ok(GeneratedField::WorkspaceId),
                            "projectId" | "project_id" => Ok(GeneratedField::ProjectId),
                            "reference" => Ok(GeneratedField::Reference),
                            "buildId" | "build_id" => Ok(GeneratedField::BuildId),
                            "commitSha" | "commit_sha" => Ok(GeneratedField::CommitSha),
                            "pullRequestNumber" | "pull_request_number" => Ok(GeneratedField::PullRequestNumber),
                            "trackedBranchId" | "tracked_branch_id" => Ok(GeneratedField::TrackedBranchId),
                            "analyzedAt" | "analyzed_at" => Ok(GeneratedField::AnalyzedAt),
                            "name" => Ok(GeneratedField::Name),
                            "fullyQualifiedName" | "fully_qualified_name" => Ok(GeneratedField::FullyQualifiedName),
                            "path" => Ok(GeneratedField::Path),
                            "kind" => Ok(GeneratedField::Kind),
                            "language" => Ok(GeneratedField::Language),
                            "files" => Ok(GeneratedField::Files),
                            "classes" => Ok(GeneratedField::Classes),
                            "functions" => Ok(GeneratedField::Functions),
                            "fields" => Ok(GeneratedField::Fields),
                            "lines" => Ok(GeneratedField::Lines),
                            "codeLines" | "code_lines" => Ok(GeneratedField::CodeLines),
                            "commentLines" | "comment_lines" => Ok(GeneratedField::CommentLines),
                            "blankLines" | "blank_lines" => Ok(GeneratedField::BlankLines),
                            "complexity" => Ok(GeneratedField::Complexity),
                            "cyclomatic" => Ok(GeneratedField::Cyclomatic),
                            "lcom4" => Ok(GeneratedField::Lcom4),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Stats;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct qlty.analysis.v1.Stats")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Stats, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut workspace_id__ = None;
                let mut project_id__ = None;
                let mut reference__ = None;
                let mut build_id__ = None;
                let mut commit_sha__ = None;
                let mut pull_request_number__ = None;
                let mut tracked_branch_id__ = None;
                let mut analyzed_at__ = None;
                let mut name__ = None;
                let mut fully_qualified_name__ = None;
                let mut path__ = None;
                let mut kind__ = None;
                let mut language__ = None;
                let mut files__ = None;
                let mut classes__ = None;
                let mut functions__ = None;
                let mut fields__ = None;
                let mut lines__ = None;
                let mut code_lines__ = None;
                let mut comment_lines__ = None;
                let mut blank_lines__ = None;
                let mut complexity__ = None;
                let mut cyclomatic__ = None;
                let mut lcom4__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::WorkspaceId => {
                            if workspace_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("workspaceId"));
                            }
                            workspace_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ProjectId => {
                            if project_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("projectId"));
                            }
                            project_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Reference => {
                            if reference__.is_some() {
                                return Err(serde::de::Error::duplicate_field("reference"));
                            }
                            reference__ = Some(map_.next_value()?);
                        }
                        GeneratedField::BuildId => {
                            if build_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("buildId"));
                            }
                            build_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::CommitSha => {
                            if commit_sha__.is_some() {
                                return Err(serde::de::Error::duplicate_field("commitSha"));
                            }
                            commit_sha__ = Some(map_.next_value()?);
                        }
                        GeneratedField::PullRequestNumber => {
                            if pull_request_number__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pullRequestNumber"));
                            }
                            pull_request_number__ = map_.next_value()?;
                        }
                        GeneratedField::TrackedBranchId => {
                            if tracked_branch_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("trackedBranchId"));
                            }
                            tracked_branch_id__ = map_.next_value()?;
                        }
                        GeneratedField::AnalyzedAt => {
                            if analyzed_at__.is_some() {
                                return Err(serde::de::Error::duplicate_field("analyzedAt"));
                            }
                            analyzed_at__ = map_.next_value()?;
                        }
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::FullyQualifiedName => {
                            if fully_qualified_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fullyQualifiedName"));
                            }
                            fully_qualified_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Path => {
                            if path__.is_some() {
                                return Err(serde::de::Error::duplicate_field("path"));
                            }
                            path__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Kind => {
                            if kind__.is_some() {
                                return Err(serde::de::Error::duplicate_field("kind"));
                            }
                            kind__ = Some(map_.next_value::<ComponentType>()? as i32);
                        }
                        GeneratedField::Language => {
                            if language__.is_some() {
                                return Err(serde::de::Error::duplicate_field("language"));
                            }
                            language__ = Some(map_.next_value::<Language>()? as i32);
                        }
                        GeneratedField::Files => {
                            if files__.is_some() {
                                return Err(serde::de::Error::duplicate_field("files"));
                            }
                            files__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::Classes => {
                            if classes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("classes"));
                            }
                            classes__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::Functions => {
                            if functions__.is_some() {
                                return Err(serde::de::Error::duplicate_field("functions"));
                            }
                            functions__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::Fields => {
                            if fields__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fields"));
                            }
                            fields__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::Lines => {
                            if lines__.is_some() {
                                return Err(serde::de::Error::duplicate_field("lines"));
                            }
                            lines__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::CodeLines => {
                            if code_lines__.is_some() {
                                return Err(serde::de::Error::duplicate_field("codeLines"));
                            }
                            code_lines__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::CommentLines => {
                            if comment_lines__.is_some() {
                                return Err(serde::de::Error::duplicate_field("commentLines"));
                            }
                            comment_lines__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::BlankLines => {
                            if blank_lines__.is_some() {
                                return Err(serde::de::Error::duplicate_field("blankLines"));
                            }
                            blank_lines__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::Complexity => {
                            if complexity__.is_some() {
                                return Err(serde::de::Error::duplicate_field("complexity"));
                            }
                            complexity__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::Cyclomatic => {
                            if cyclomatic__.is_some() {
                                return Err(serde::de::Error::duplicate_field("cyclomatic"));
                            }
                            cyclomatic__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::Lcom4 => {
                            if lcom4__.is_some() {
                                return Err(serde::de::Error::duplicate_field("lcom4"));
                            }
                            lcom4__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                    }
                }
                Ok(Stats {
                    workspace_id: workspace_id__.unwrap_or_default(),
                    project_id: project_id__.unwrap_or_default(),
                    reference: reference__.unwrap_or_default(),
                    build_id: build_id__.unwrap_or_default(),
                    commit_sha: commit_sha__.unwrap_or_default(),
                    pull_request_number: pull_request_number__,
                    tracked_branch_id: tracked_branch_id__,
                    analyzed_at: analyzed_at__,
                    name: name__.unwrap_or_default(),
                    fully_qualified_name: fully_qualified_name__.unwrap_or_default(),
                    path: path__.unwrap_or_default(),
                    kind: kind__.unwrap_or_default(),
                    language: language__.unwrap_or_default(),
                    files: files__,
                    classes: classes__,
                    functions: functions__,
                    fields: fields__,
                    lines: lines__,
                    code_lines: code_lines__,
                    comment_lines: comment_lines__,
                    blank_lines: blank_lines__,
                    complexity: complexity__,
                    cyclomatic: cyclomatic__,
                    lcom4: lcom4__,
                })
            }
        }
        deserializer.deserialize_struct("qlty.analysis.v1.Stats", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Suggestion {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.id.is_empty() {
            len += 1;
        }
        if !self.description.is_empty() {
            len += 1;
        }
        if !self.patch.is_empty() {
            len += 1;
        }
        if self.r#unsafe {
            len += 1;
        }
        if self.source != 0 {
            len += 1;
        }
        if !self.replacements.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("qlty.analysis.v1.Suggestion", len)?;
        if !self.id.is_empty() {
            struct_ser.serialize_field("id", &self.id)?;
        }
        if !self.description.is_empty() {
            struct_ser.serialize_field("description", &self.description)?;
        }
        if !self.patch.is_empty() {
            struct_ser.serialize_field("patch", &self.patch)?;
        }
        if self.r#unsafe {
            struct_ser.serialize_field("unsafe", &self.r#unsafe)?;
        }
        if self.source != 0 {
            let v = SuggestionSource::try_from(self.source)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.source)))?;
            struct_ser.serialize_field("source", &v)?;
        }
        if !self.replacements.is_empty() {
            struct_ser.serialize_field("replacements", &self.replacements)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Suggestion {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "id",
            "description",
            "patch",
            "unsafe",
            "source",
            "replacements",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Id,
            Description,
            Patch,
            Unsafe,
            Source,
            Replacements,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "id" => Ok(GeneratedField::Id),
                            "description" => Ok(GeneratedField::Description),
                            "patch" => Ok(GeneratedField::Patch),
                            "unsafe" => Ok(GeneratedField::Unsafe),
                            "source" => Ok(GeneratedField::Source),
                            "replacements" => Ok(GeneratedField::Replacements),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Suggestion;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct qlty.analysis.v1.Suggestion")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Suggestion, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut id__ = None;
                let mut description__ = None;
                let mut patch__ = None;
                let mut r#unsafe__ = None;
                let mut source__ = None;
                let mut replacements__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Id => {
                            if id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Description => {
                            if description__.is_some() {
                                return Err(serde::de::Error::duplicate_field("description"));
                            }
                            description__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Patch => {
                            if patch__.is_some() {
                                return Err(serde::de::Error::duplicate_field("patch"));
                            }
                            patch__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Unsafe => {
                            if r#unsafe__.is_some() {
                                return Err(serde::de::Error::duplicate_field("unsafe"));
                            }
                            r#unsafe__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Source => {
                            if source__.is_some() {
                                return Err(serde::de::Error::duplicate_field("source"));
                            }
                            source__ = Some(map_.next_value::<SuggestionSource>()? as i32);
                        }
                        GeneratedField::Replacements => {
                            if replacements__.is_some() {
                                return Err(serde::de::Error::duplicate_field("replacements"));
                            }
                            replacements__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(Suggestion {
                    id: id__.unwrap_or_default(),
                    description: description__.unwrap_or_default(),
                    patch: patch__.unwrap_or_default(),
                    r#unsafe: r#unsafe__.unwrap_or_default(),
                    source: source__.unwrap_or_default(),
                    replacements: replacements__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("qlty.analysis.v1.Suggestion", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SuggestionSource {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "SUGGESTION_SOURCE_UNSPECIFIED",
            Self::Tool => "SUGGESTION_SOURCE_TOOL",
            Self::Llm => "SUGGESTION_SOURCE_LLM",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for SuggestionSource {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "SUGGESTION_SOURCE_UNSPECIFIED",
            "SUGGESTION_SOURCE_TOOL",
            "SUGGESTION_SOURCE_LLM",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SuggestionSource;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "SUGGESTION_SOURCE_UNSPECIFIED" => Ok(SuggestionSource::Unspecified),
                    "SUGGESTION_SOURCE_TOOL" => Ok(SuggestionSource::Tool),
                    "SUGGESTION_SOURCE_LLM" => Ok(SuggestionSource::Llm),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
