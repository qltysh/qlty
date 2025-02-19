// @generated
impl serde::Serialize for CoverageMetadata {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.upload_id.is_empty() {
            len += 1;
        }
        if self.project_id.is_some() {
            len += 1;
        }
        if !self.build_id.is_empty() {
            len += 1;
        }
        if !self.ci.is_empty() {
            len += 1;
        }
        if !self.ci_url.is_empty() {
            len += 1;
        }
        if !self.repository_web_url.is_empty() {
            len += 1;
        }
        if !self.repository_origin_url.is_empty() {
            len += 1;
        }
        if !self.branch.is_empty() {
            len += 1;
        }
        if !self.workflow.is_empty() {
            len += 1;
        }
        if !self.job.is_empty() {
            len += 1;
        }
        if !self.run.is_empty() {
            len += 1;
        }
        if !self.run_url.is_empty() {
            len += 1;
        }
        if !self.commit_sha.is_empty() {
            len += 1;
        }
        if !self.commit_headline.is_empty() {
            len += 1;
        }
        if !self.commit_message.is_empty() {
            len += 1;
        }
        if !self.author_name.is_empty() {
            len += 1;
        }
        if !self.author_email.is_empty() {
            len += 1;
        }
        if self.author_time.is_some() {
            len += 1;
        }
        if !self.committer_name.is_empty() {
            len += 1;
        }
        if !self.committer_email.is_empty() {
            len += 1;
        }
        if self.commit_time.is_some() {
            len += 1;
        }
        if !self.pull_request_number.is_empty() {
            len += 1;
        }
        if !self.pull_request_url.is_empty() {
            len += 1;
        }
        if !self.head_ref.is_empty() {
            len += 1;
        }
        if !self.head_commit.is_empty() {
            len += 1;
        }
        if !self.base_ref.is_empty() {
            len += 1;
        }
        if !self.base_commit.is_empty() {
            len += 1;
        }
        if self.tag.is_some() {
            len += 1;
        }
        if !self.description.is_empty() {
            len += 1;
        }
        if self.uploaded_at.is_some() {
            len += 1;
        }
        if !self.cli_version.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("qlty.tests.v1.CoverageMetadata", len)?;
        if !self.upload_id.is_empty() {
            struct_ser.serialize_field("uploadId", &self.upload_id)?;
        }
        if let Some(v) = self.project_id.as_ref() {
            struct_ser.serialize_field("projectId", v)?;
        }
        if !self.build_id.is_empty() {
            struct_ser.serialize_field("buildId", &self.build_id)?;
        }
        if !self.ci.is_empty() {
            struct_ser.serialize_field("ci", &self.ci)?;
        }
        if !self.ci_url.is_empty() {
            struct_ser.serialize_field("ciUrl", &self.ci_url)?;
        }
        if !self.repository_web_url.is_empty() {
            struct_ser.serialize_field("repositoryWebUrl", &self.repository_web_url)?;
        }
        if !self.repository_origin_url.is_empty() {
            struct_ser.serialize_field("repositoryOriginUrl", &self.repository_origin_url)?;
        }
        if !self.branch.is_empty() {
            struct_ser.serialize_field("branch", &self.branch)?;
        }
        if !self.workflow.is_empty() {
            struct_ser.serialize_field("workflow", &self.workflow)?;
        }
        if !self.job.is_empty() {
            struct_ser.serialize_field("job", &self.job)?;
        }
        if !self.run.is_empty() {
            struct_ser.serialize_field("run", &self.run)?;
        }
        if !self.run_url.is_empty() {
            struct_ser.serialize_field("runUrl", &self.run_url)?;
        }
        if !self.commit_sha.is_empty() {
            struct_ser.serialize_field("commitSha", &self.commit_sha)?;
        }
        if !self.commit_headline.is_empty() {
            struct_ser.serialize_field("commitHeadline", &self.commit_headline)?;
        }
        if !self.commit_message.is_empty() {
            struct_ser.serialize_field("commitMessage", &self.commit_message)?;
        }
        if !self.author_name.is_empty() {
            struct_ser.serialize_field("authorName", &self.author_name)?;
        }
        if !self.author_email.is_empty() {
            struct_ser.serialize_field("authorEmail", &self.author_email)?;
        }
        if let Some(v) = self.author_time.as_ref() {
            struct_ser.serialize_field("authorTime", v)?;
        }
        if !self.committer_name.is_empty() {
            struct_ser.serialize_field("committerName", &self.committer_name)?;
        }
        if !self.committer_email.is_empty() {
            struct_ser.serialize_field("committerEmail", &self.committer_email)?;
        }
        if let Some(v) = self.commit_time.as_ref() {
            struct_ser.serialize_field("commitTime", v)?;
        }
        if !self.pull_request_number.is_empty() {
            struct_ser.serialize_field("pullRequestNumber", &self.pull_request_number)?;
        }
        if !self.pull_request_url.is_empty() {
            struct_ser.serialize_field("pullRequestUrl", &self.pull_request_url)?;
        }
        if !self.head_ref.is_empty() {
            struct_ser.serialize_field("headRef", &self.head_ref)?;
        }
        if !self.head_commit.is_empty() {
            struct_ser.serialize_field("headCommit", &self.head_commit)?;
        }
        if !self.base_ref.is_empty() {
            struct_ser.serialize_field("baseRef", &self.base_ref)?;
        }
        if !self.base_commit.is_empty() {
            struct_ser.serialize_field("baseCommit", &self.base_commit)?;
        }
        if let Some(v) = self.tag.as_ref() {
            struct_ser.serialize_field("tag", v)?;
        }
        if !self.description.is_empty() {
            struct_ser.serialize_field("description", &self.description)?;
        }
        if let Some(v) = self.uploaded_at.as_ref() {
            struct_ser.serialize_field("uploadedAt", v)?;
        }
        if !self.cli_version.is_empty() {
            struct_ser.serialize_field("cliVersion", &self.cli_version)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CoverageMetadata {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "upload_id",
            "uploadId",
            "project_id",
            "projectId",
            "build_id",
            "buildId",
            "ci",
            "ci_url",
            "ciUrl",
            "repository_web_url",
            "repositoryWebUrl",
            "repository_origin_url",
            "repositoryOriginUrl",
            "branch",
            "workflow",
            "job",
            "run",
            "run_url",
            "runUrl",
            "commit_sha",
            "commitSha",
            "commit_headline",
            "commitHeadline",
            "commit_message",
            "commitMessage",
            "author_name",
            "authorName",
            "author_email",
            "authorEmail",
            "author_time",
            "authorTime",
            "committer_name",
            "committerName",
            "committer_email",
            "committerEmail",
            "commit_time",
            "commitTime",
            "pull_request_number",
            "pullRequestNumber",
            "pull_request_url",
            "pullRequestUrl",
            "head_ref",
            "headRef",
            "head_commit",
            "headCommit",
            "base_ref",
            "baseRef",
            "base_commit",
            "baseCommit",
            "tag",
            "description",
            "uploaded_at",
            "uploadedAt",
            "cli_version",
            "cliVersion",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            UploadId,
            ProjectId,
            BuildId,
            Ci,
            CiUrl,
            RepositoryWebUrl,
            RepositoryOriginUrl,
            Branch,
            Workflow,
            Job,
            Run,
            RunUrl,
            CommitSha,
            CommitHeadline,
            CommitMessage,
            AuthorName,
            AuthorEmail,
            AuthorTime,
            CommitterName,
            CommitterEmail,
            CommitTime,
            PullRequestNumber,
            PullRequestUrl,
            HeadRef,
            HeadCommit,
            BaseRef,
            BaseCommit,
            Tag,
            Description,
            UploadedAt,
            CliVersion,
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
                            "uploadId" | "upload_id" => Ok(GeneratedField::UploadId),
                            "projectId" | "project_id" => Ok(GeneratedField::ProjectId),
                            "buildId" | "build_id" => Ok(GeneratedField::BuildId),
                            "ci" => Ok(GeneratedField::Ci),
                            "ciUrl" | "ci_url" => Ok(GeneratedField::CiUrl),
                            "repositoryWebUrl" | "repository_web_url" => Ok(GeneratedField::RepositoryWebUrl),
                            "repositoryOriginUrl" | "repository_origin_url" => Ok(GeneratedField::RepositoryOriginUrl),
                            "branch" => Ok(GeneratedField::Branch),
                            "workflow" => Ok(GeneratedField::Workflow),
                            "job" => Ok(GeneratedField::Job),
                            "run" => Ok(GeneratedField::Run),
                            "runUrl" | "run_url" => Ok(GeneratedField::RunUrl),
                            "commitSha" | "commit_sha" => Ok(GeneratedField::CommitSha),
                            "commitHeadline" | "commit_headline" => Ok(GeneratedField::CommitHeadline),
                            "commitMessage" | "commit_message" => Ok(GeneratedField::CommitMessage),
                            "authorName" | "author_name" => Ok(GeneratedField::AuthorName),
                            "authorEmail" | "author_email" => Ok(GeneratedField::AuthorEmail),
                            "authorTime" | "author_time" => Ok(GeneratedField::AuthorTime),
                            "committerName" | "committer_name" => Ok(GeneratedField::CommitterName),
                            "committerEmail" | "committer_email" => Ok(GeneratedField::CommitterEmail),
                            "commitTime" | "commit_time" => Ok(GeneratedField::CommitTime),
                            "pullRequestNumber" | "pull_request_number" => Ok(GeneratedField::PullRequestNumber),
                            "pullRequestUrl" | "pull_request_url" => Ok(GeneratedField::PullRequestUrl),
                            "headRef" | "head_ref" => Ok(GeneratedField::HeadRef),
                            "headCommit" | "head_commit" => Ok(GeneratedField::HeadCommit),
                            "baseRef" | "base_ref" => Ok(GeneratedField::BaseRef),
                            "baseCommit" | "base_commit" => Ok(GeneratedField::BaseCommit),
                            "tag" => Ok(GeneratedField::Tag),
                            "description" => Ok(GeneratedField::Description),
                            "uploadedAt" | "uploaded_at" => Ok(GeneratedField::UploadedAt),
                            "cliVersion" | "cli_version" => Ok(GeneratedField::CliVersion),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CoverageMetadata;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct qlty.tests.v1.CoverageMetadata")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CoverageMetadata, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut upload_id__ = None;
                let mut project_id__ = None;
                let mut build_id__ = None;
                let mut ci__ = None;
                let mut ci_url__ = None;
                let mut repository_web_url__ = None;
                let mut repository_origin_url__ = None;
                let mut branch__ = None;
                let mut workflow__ = None;
                let mut job__ = None;
                let mut run__ = None;
                let mut run_url__ = None;
                let mut commit_sha__ = None;
                let mut commit_headline__ = None;
                let mut commit_message__ = None;
                let mut author_name__ = None;
                let mut author_email__ = None;
                let mut author_time__ = None;
                let mut committer_name__ = None;
                let mut committer_email__ = None;
                let mut commit_time__ = None;
                let mut pull_request_number__ = None;
                let mut pull_request_url__ = None;
                let mut head_ref__ = None;
                let mut head_commit__ = None;
                let mut base_ref__ = None;
                let mut base_commit__ = None;
                let mut tag__ = None;
                let mut description__ = None;
                let mut uploaded_at__ = None;
                let mut cli_version__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::UploadId => {
                            if upload_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("uploadId"));
                            }
                            upload_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ProjectId => {
                            if project_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("projectId"));
                            }
                            project_id__ = map_.next_value()?;
                        }
                        GeneratedField::BuildId => {
                            if build_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("buildId"));
                            }
                            build_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Ci => {
                            if ci__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ci"));
                            }
                            ci__ = Some(map_.next_value()?);
                        }
                        GeneratedField::CiUrl => {
                            if ci_url__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ciUrl"));
                            }
                            ci_url__ = Some(map_.next_value()?);
                        }
                        GeneratedField::RepositoryWebUrl => {
                            if repository_web_url__.is_some() {
                                return Err(serde::de::Error::duplicate_field("repositoryWebUrl"));
                            }
                            repository_web_url__ = Some(map_.next_value()?);
                        }
                        GeneratedField::RepositoryOriginUrl => {
                            if repository_origin_url__.is_some() {
                                return Err(serde::de::Error::duplicate_field("repositoryOriginUrl"));
                            }
                            repository_origin_url__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Branch => {
                            if branch__.is_some() {
                                return Err(serde::de::Error::duplicate_field("branch"));
                            }
                            branch__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Workflow => {
                            if workflow__.is_some() {
                                return Err(serde::de::Error::duplicate_field("workflow"));
                            }
                            workflow__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Job => {
                            if job__.is_some() {
                                return Err(serde::de::Error::duplicate_field("job"));
                            }
                            job__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Run => {
                            if run__.is_some() {
                                return Err(serde::de::Error::duplicate_field("run"));
                            }
                            run__ = Some(map_.next_value()?);
                        }
                        GeneratedField::RunUrl => {
                            if run_url__.is_some() {
                                return Err(serde::de::Error::duplicate_field("runUrl"));
                            }
                            run_url__ = Some(map_.next_value()?);
                        }
                        GeneratedField::CommitSha => {
                            if commit_sha__.is_some() {
                                return Err(serde::de::Error::duplicate_field("commitSha"));
                            }
                            commit_sha__ = Some(map_.next_value()?);
                        }
                        GeneratedField::CommitHeadline => {
                            if commit_headline__.is_some() {
                                return Err(serde::de::Error::duplicate_field("commitHeadline"));
                            }
                            commit_headline__ = Some(map_.next_value()?);
                        }
                        GeneratedField::CommitMessage => {
                            if commit_message__.is_some() {
                                return Err(serde::de::Error::duplicate_field("commitMessage"));
                            }
                            commit_message__ = Some(map_.next_value()?);
                        }
                        GeneratedField::AuthorName => {
                            if author_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("authorName"));
                            }
                            author_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::AuthorEmail => {
                            if author_email__.is_some() {
                                return Err(serde::de::Error::duplicate_field("authorEmail"));
                            }
                            author_email__ = Some(map_.next_value()?);
                        }
                        GeneratedField::AuthorTime => {
                            if author_time__.is_some() {
                                return Err(serde::de::Error::duplicate_field("authorTime"));
                            }
                            author_time__ = map_.next_value()?;
                        }
                        GeneratedField::CommitterName => {
                            if committer_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("committerName"));
                            }
                            committer_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::CommitterEmail => {
                            if committer_email__.is_some() {
                                return Err(serde::de::Error::duplicate_field("committerEmail"));
                            }
                            committer_email__ = Some(map_.next_value()?);
                        }
                        GeneratedField::CommitTime => {
                            if commit_time__.is_some() {
                                return Err(serde::de::Error::duplicate_field("commitTime"));
                            }
                            commit_time__ = map_.next_value()?;
                        }
                        GeneratedField::PullRequestNumber => {
                            if pull_request_number__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pullRequestNumber"));
                            }
                            pull_request_number__ = Some(map_.next_value()?);
                        }
                        GeneratedField::PullRequestUrl => {
                            if pull_request_url__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pullRequestUrl"));
                            }
                            pull_request_url__ = Some(map_.next_value()?);
                        }
                        GeneratedField::HeadRef => {
                            if head_ref__.is_some() {
                                return Err(serde::de::Error::duplicate_field("headRef"));
                            }
                            head_ref__ = Some(map_.next_value()?);
                        }
                        GeneratedField::HeadCommit => {
                            if head_commit__.is_some() {
                                return Err(serde::de::Error::duplicate_field("headCommit"));
                            }
                            head_commit__ = Some(map_.next_value()?);
                        }
                        GeneratedField::BaseRef => {
                            if base_ref__.is_some() {
                                return Err(serde::de::Error::duplicate_field("baseRef"));
                            }
                            base_ref__ = Some(map_.next_value()?);
                        }
                        GeneratedField::BaseCommit => {
                            if base_commit__.is_some() {
                                return Err(serde::de::Error::duplicate_field("baseCommit"));
                            }
                            base_commit__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Tag => {
                            if tag__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tag"));
                            }
                            tag__ = map_.next_value()?;
                        }
                        GeneratedField::Description => {
                            if description__.is_some() {
                                return Err(serde::de::Error::duplicate_field("description"));
                            }
                            description__ = Some(map_.next_value()?);
                        }
                        GeneratedField::UploadedAt => {
                            if uploaded_at__.is_some() {
                                return Err(serde::de::Error::duplicate_field("uploadedAt"));
                            }
                            uploaded_at__ = map_.next_value()?;
                        }
                        GeneratedField::CliVersion => {
                            if cli_version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("cliVersion"));
                            }
                            cli_version__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(CoverageMetadata {
                    upload_id: upload_id__.unwrap_or_default(),
                    project_id: project_id__,
                    build_id: build_id__.unwrap_or_default(),
                    ci: ci__.unwrap_or_default(),
                    ci_url: ci_url__.unwrap_or_default(),
                    repository_web_url: repository_web_url__.unwrap_or_default(),
                    repository_origin_url: repository_origin_url__.unwrap_or_default(),
                    branch: branch__.unwrap_or_default(),
                    workflow: workflow__.unwrap_or_default(),
                    job: job__.unwrap_or_default(),
                    run: run__.unwrap_or_default(),
                    run_url: run_url__.unwrap_or_default(),
                    commit_sha: commit_sha__.unwrap_or_default(),
                    commit_headline: commit_headline__.unwrap_or_default(),
                    commit_message: commit_message__.unwrap_or_default(),
                    author_name: author_name__.unwrap_or_default(),
                    author_email: author_email__.unwrap_or_default(),
                    author_time: author_time__,
                    committer_name: committer_name__.unwrap_or_default(),
                    committer_email: committer_email__.unwrap_or_default(),
                    commit_time: commit_time__,
                    pull_request_number: pull_request_number__.unwrap_or_default(),
                    pull_request_url: pull_request_url__.unwrap_or_default(),
                    head_ref: head_ref__.unwrap_or_default(),
                    head_commit: head_commit__.unwrap_or_default(),
                    base_ref: base_ref__.unwrap_or_default(),
                    base_commit: base_commit__.unwrap_or_default(),
                    tag: tag__,
                    description: description__.unwrap_or_default(),
                    uploaded_at: uploaded_at__,
                    cli_version: cli_version__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("qlty.tests.v1.CoverageMetadata", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CoverageSummary {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.covered != 0 {
            len += 1;
        }
        if self.missed != 0 {
            len += 1;
        }
        if self.omit != 0 {
            len += 1;
        }
        if self.total != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("qlty.tests.v1.CoverageSummary", len)?;
        if self.covered != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("covered", ToString::to_string(&self.covered).as_str())?;
        }
        if self.missed != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("missed", ToString::to_string(&self.missed).as_str())?;
        }
        if self.omit != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("omit", ToString::to_string(&self.omit).as_str())?;
        }
        if self.total != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("total", ToString::to_string(&self.total).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CoverageSummary {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "covered",
            "missed",
            "omit",
            "total",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Covered,
            Missed,
            Omit,
            Total,
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
                            "covered" => Ok(GeneratedField::Covered),
                            "missed" => Ok(GeneratedField::Missed),
                            "omit" => Ok(GeneratedField::Omit),
                            "total" => Ok(GeneratedField::Total),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CoverageSummary;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct qlty.tests.v1.CoverageSummary")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CoverageSummary, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut covered__ = None;
                let mut missed__ = None;
                let mut omit__ = None;
                let mut total__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Covered => {
                            if covered__.is_some() {
                                return Err(serde::de::Error::duplicate_field("covered"));
                            }
                            covered__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Missed => {
                            if missed__.is_some() {
                                return Err(serde::de::Error::duplicate_field("missed"));
                            }
                            missed__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Omit => {
                            if omit__.is_some() {
                                return Err(serde::de::Error::duplicate_field("omit"));
                            }
                            omit__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Total => {
                            if total__.is_some() {
                                return Err(serde::de::Error::duplicate_field("total"));
                            }
                            total__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(CoverageSummary {
                    covered: covered__.unwrap_or_default(),
                    missed: missed__.unwrap_or_default(),
                    omit: omit__.unwrap_or_default(),
                    total: total__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("qlty.tests.v1.CoverageSummary", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for FileCoverage {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.upload_id.is_empty() {
            len += 1;
        }
        if !self.build_id.is_empty() {
            len += 1;
        }
        if !self.report_id.is_empty() {
            len += 1;
        }
        if !self.path.is_empty() {
            len += 1;
        }
        if !self.blob_oid.is_empty() {
            len += 1;
        }
        if !self.contents_md5.is_empty() {
            len += 1;
        }
        if self.summary.is_some() {
            len += 1;
        }
        if !self.hits.is_empty() {
            len += 1;
        }
        if self.project_id.is_some() {
            len += 1;
        }
        if self.tag.is_some() {
            len += 1;
        }
        if self.commit_sha.is_some() {
            len += 1;
        }
        if self.uploaded_at.is_some() {
            len += 1;
        }
        if !self.branch.is_empty() {
            len += 1;
        }
        if self.pull_request_number.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("qlty.tests.v1.FileCoverage", len)?;
        if !self.upload_id.is_empty() {
            struct_ser.serialize_field("uploadId", &self.upload_id)?;
        }
        if !self.build_id.is_empty() {
            struct_ser.serialize_field("buildId", &self.build_id)?;
        }
        if !self.report_id.is_empty() {
            struct_ser.serialize_field("reportId", &self.report_id)?;
        }
        if !self.path.is_empty() {
            struct_ser.serialize_field("path", &self.path)?;
        }
        if !self.blob_oid.is_empty() {
            struct_ser.serialize_field("blobOid", &self.blob_oid)?;
        }
        if !self.contents_md5.is_empty() {
            struct_ser.serialize_field("contentsMd5", &self.contents_md5)?;
        }
        if let Some(v) = self.summary.as_ref() {
            struct_ser.serialize_field("summary", v)?;
        }
        if !self.hits.is_empty() {
            struct_ser.serialize_field("hits", &self.hits.iter().map(ToString::to_string).collect::<Vec<_>>())?;
        }
        if let Some(v) = self.project_id.as_ref() {
            struct_ser.serialize_field("projectId", v)?;
        }
        if let Some(v) = self.tag.as_ref() {
            struct_ser.serialize_field("tag", v)?;
        }
        if let Some(v) = self.commit_sha.as_ref() {
            struct_ser.serialize_field("commitSha", v)?;
        }
        if let Some(v) = self.uploaded_at.as_ref() {
            struct_ser.serialize_field("uploadedAt", v)?;
        }
        if !self.branch.is_empty() {
            struct_ser.serialize_field("branch", &self.branch)?;
        }
        if let Some(v) = self.pull_request_number.as_ref() {
            struct_ser.serialize_field("pullRequestNumber", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for FileCoverage {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "upload_id",
            "uploadId",
            "build_id",
            "buildId",
            "report_id",
            "reportId",
            "path",
            "blob_oid",
            "blobOid",
            "contents_md5",
            "contentsMd5",
            "summary",
            "hits",
            "project_id",
            "projectId",
            "tag",
            "commit_sha",
            "commitSha",
            "uploaded_at",
            "uploadedAt",
            "branch",
            "pull_request_number",
            "pullRequestNumber",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            UploadId,
            BuildId,
            ReportId,
            Path,
            BlobOid,
            ContentsMd5,
            Summary,
            Hits,
            ProjectId,
            Tag,
            CommitSha,
            UploadedAt,
            Branch,
            PullRequestNumber,
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
                            "uploadId" | "upload_id" => Ok(GeneratedField::UploadId),
                            "buildId" | "build_id" => Ok(GeneratedField::BuildId),
                            "reportId" | "report_id" => Ok(GeneratedField::ReportId),
                            "path" => Ok(GeneratedField::Path),
                            "blobOid" | "blob_oid" => Ok(GeneratedField::BlobOid),
                            "contentsMd5" | "contents_md5" => Ok(GeneratedField::ContentsMd5),
                            "summary" => Ok(GeneratedField::Summary),
                            "hits" => Ok(GeneratedField::Hits),
                            "projectId" | "project_id" => Ok(GeneratedField::ProjectId),
                            "tag" => Ok(GeneratedField::Tag),
                            "commitSha" | "commit_sha" => Ok(GeneratedField::CommitSha),
                            "uploadedAt" | "uploaded_at" => Ok(GeneratedField::UploadedAt),
                            "branch" => Ok(GeneratedField::Branch),
                            "pullRequestNumber" | "pull_request_number" => Ok(GeneratedField::PullRequestNumber),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = FileCoverage;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct qlty.tests.v1.FileCoverage")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<FileCoverage, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut upload_id__ = None;
                let mut build_id__ = None;
                let mut report_id__ = None;
                let mut path__ = None;
                let mut blob_oid__ = None;
                let mut contents_md5__ = None;
                let mut summary__ = None;
                let mut hits__ = None;
                let mut project_id__ = None;
                let mut tag__ = None;
                let mut commit_sha__ = None;
                let mut uploaded_at__ = None;
                let mut branch__ = None;
                let mut pull_request_number__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::UploadId => {
                            if upload_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("uploadId"));
                            }
                            upload_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::BuildId => {
                            if build_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("buildId"));
                            }
                            build_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ReportId => {
                            if report_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("reportId"));
                            }
                            report_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Path => {
                            if path__.is_some() {
                                return Err(serde::de::Error::duplicate_field("path"));
                            }
                            path__ = Some(map_.next_value()?);
                        }
                        GeneratedField::BlobOid => {
                            if blob_oid__.is_some() {
                                return Err(serde::de::Error::duplicate_field("blobOid"));
                            }
                            blob_oid__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ContentsMd5 => {
                            if contents_md5__.is_some() {
                                return Err(serde::de::Error::duplicate_field("contentsMd5"));
                            }
                            contents_md5__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Summary => {
                            if summary__.is_some() {
                                return Err(serde::de::Error::duplicate_field("summary"));
                            }
                            summary__ = map_.next_value()?;
                        }
                        GeneratedField::Hits => {
                            if hits__.is_some() {
                                return Err(serde::de::Error::duplicate_field("hits"));
                            }
                            hits__ = 
                                Some(map_.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect())
                            ;
                        }
                        GeneratedField::ProjectId => {
                            if project_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("projectId"));
                            }
                            project_id__ = map_.next_value()?;
                        }
                        GeneratedField::Tag => {
                            if tag__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tag"));
                            }
                            tag__ = map_.next_value()?;
                        }
                        GeneratedField::CommitSha => {
                            if commit_sha__.is_some() {
                                return Err(serde::de::Error::duplicate_field("commitSha"));
                            }
                            commit_sha__ = map_.next_value()?;
                        }
                        GeneratedField::UploadedAt => {
                            if uploaded_at__.is_some() {
                                return Err(serde::de::Error::duplicate_field("uploadedAt"));
                            }
                            uploaded_at__ = map_.next_value()?;
                        }
                        GeneratedField::Branch => {
                            if branch__.is_some() {
                                return Err(serde::de::Error::duplicate_field("branch"));
                            }
                            branch__ = Some(map_.next_value()?);
                        }
                        GeneratedField::PullRequestNumber => {
                            if pull_request_number__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pullRequestNumber"));
                            }
                            pull_request_number__ = map_.next_value()?;
                        }
                    }
                }
                Ok(FileCoverage {
                    upload_id: upload_id__.unwrap_or_default(),
                    build_id: build_id__.unwrap_or_default(),
                    report_id: report_id__.unwrap_or_default(),
                    path: path__.unwrap_or_default(),
                    blob_oid: blob_oid__.unwrap_or_default(),
                    contents_md5: contents_md5__.unwrap_or_default(),
                    summary: summary__,
                    hits: hits__.unwrap_or_default(),
                    project_id: project_id__,
                    tag: tag__,
                    commit_sha: commit_sha__,
                    uploaded_at: uploaded_at__,
                    branch: branch__.unwrap_or_default(),
                    pull_request_number: pull_request_number__,
                })
            }
        }
        deserializer.deserialize_struct("qlty.tests.v1.FileCoverage", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ReportFile {
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
        if !self.build_id.is_empty() {
            len += 1;
        }
        if !self.tool.is_empty() {
            len += 1;
        }
        if !self.format.is_empty() {
            len += 1;
        }
        if !self.path.is_empty() {
            len += 1;
        }
        if !self.language.is_empty() {
            len += 1;
        }
        if !self.contents_md5.is_empty() {
            len += 1;
        }
        if self.size != 0 {
            len += 1;
        }
        if self.project_id.is_some() {
            len += 1;
        }
        if self.tag.is_some() {
            len += 1;
        }
        if self.commit_sha.is_some() {
            len += 1;
        }
        if self.uploaded_at.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("qlty.tests.v1.ReportFile", len)?;
        if !self.id.is_empty() {
            struct_ser.serialize_field("id", &self.id)?;
        }
        if !self.build_id.is_empty() {
            struct_ser.serialize_field("buildId", &self.build_id)?;
        }
        if !self.tool.is_empty() {
            struct_ser.serialize_field("tool", &self.tool)?;
        }
        if !self.format.is_empty() {
            struct_ser.serialize_field("format", &self.format)?;
        }
        if !self.path.is_empty() {
            struct_ser.serialize_field("path", &self.path)?;
        }
        if !self.language.is_empty() {
            struct_ser.serialize_field("language", &self.language)?;
        }
        if !self.contents_md5.is_empty() {
            struct_ser.serialize_field("contentsMd5", &self.contents_md5)?;
        }
        if self.size != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("size", ToString::to_string(&self.size).as_str())?;
        }
        if let Some(v) = self.project_id.as_ref() {
            struct_ser.serialize_field("projectId", v)?;
        }
        if let Some(v) = self.tag.as_ref() {
            struct_ser.serialize_field("tag", v)?;
        }
        if let Some(v) = self.commit_sha.as_ref() {
            struct_ser.serialize_field("commitSha", v)?;
        }
        if let Some(v) = self.uploaded_at.as_ref() {
            struct_ser.serialize_field("uploadedAt", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ReportFile {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "id",
            "build_id",
            "buildId",
            "tool",
            "format",
            "path",
            "language",
            "contents_md5",
            "contentsMd5",
            "size",
            "project_id",
            "projectId",
            "tag",
            "commit_sha",
            "commitSha",
            "uploaded_at",
            "uploadedAt",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Id,
            BuildId,
            Tool,
            Format,
            Path,
            Language,
            ContentsMd5,
            Size,
            ProjectId,
            Tag,
            CommitSha,
            UploadedAt,
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
                            "buildId" | "build_id" => Ok(GeneratedField::BuildId),
                            "tool" => Ok(GeneratedField::Tool),
                            "format" => Ok(GeneratedField::Format),
                            "path" => Ok(GeneratedField::Path),
                            "language" => Ok(GeneratedField::Language),
                            "contentsMd5" | "contents_md5" => Ok(GeneratedField::ContentsMd5),
                            "size" => Ok(GeneratedField::Size),
                            "projectId" | "project_id" => Ok(GeneratedField::ProjectId),
                            "tag" => Ok(GeneratedField::Tag),
                            "commitSha" | "commit_sha" => Ok(GeneratedField::CommitSha),
                            "uploadedAt" | "uploaded_at" => Ok(GeneratedField::UploadedAt),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ReportFile;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct qlty.tests.v1.ReportFile")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ReportFile, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut id__ = None;
                let mut build_id__ = None;
                let mut tool__ = None;
                let mut format__ = None;
                let mut path__ = None;
                let mut language__ = None;
                let mut contents_md5__ = None;
                let mut size__ = None;
                let mut project_id__ = None;
                let mut tag__ = None;
                let mut commit_sha__ = None;
                let mut uploaded_at__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Id => {
                            if id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::BuildId => {
                            if build_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("buildId"));
                            }
                            build_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Tool => {
                            if tool__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tool"));
                            }
                            tool__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Format => {
                            if format__.is_some() {
                                return Err(serde::de::Error::duplicate_field("format"));
                            }
                            format__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Path => {
                            if path__.is_some() {
                                return Err(serde::de::Error::duplicate_field("path"));
                            }
                            path__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Language => {
                            if language__.is_some() {
                                return Err(serde::de::Error::duplicate_field("language"));
                            }
                            language__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ContentsMd5 => {
                            if contents_md5__.is_some() {
                                return Err(serde::de::Error::duplicate_field("contentsMd5"));
                            }
                            contents_md5__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Size => {
                            if size__.is_some() {
                                return Err(serde::de::Error::duplicate_field("size"));
                            }
                            size__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ProjectId => {
                            if project_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("projectId"));
                            }
                            project_id__ = map_.next_value()?;
                        }
                        GeneratedField::Tag => {
                            if tag__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tag"));
                            }
                            tag__ = map_.next_value()?;
                        }
                        GeneratedField::CommitSha => {
                            if commit_sha__.is_some() {
                                return Err(serde::de::Error::duplicate_field("commitSha"));
                            }
                            commit_sha__ = map_.next_value()?;
                        }
                        GeneratedField::UploadedAt => {
                            if uploaded_at__.is_some() {
                                return Err(serde::de::Error::duplicate_field("uploadedAt"));
                            }
                            uploaded_at__ = map_.next_value()?;
                        }
                    }
                }
                Ok(ReportFile {
                    id: id__.unwrap_or_default(),
                    build_id: build_id__.unwrap_or_default(),
                    tool: tool__.unwrap_or_default(),
                    format: format__.unwrap_or_default(),
                    path: path__.unwrap_or_default(),
                    language: language__.unwrap_or_default(),
                    contents_md5: contents_md5__.unwrap_or_default(),
                    size: size__.unwrap_or_default(),
                    project_id: project_id__,
                    tag: tag__,
                    commit_sha: commit_sha__,
                    uploaded_at: uploaded_at__,
                })
            }
        }
        deserializer.deserialize_struct("qlty.tests.v1.ReportFile", FIELDS, GeneratedVisitor)
    }
}
