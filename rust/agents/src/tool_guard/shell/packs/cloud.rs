//! Cloud packs: `cloud.aws`, `cloud.gcp`, `cloud.azure`, `cloud.iac`.

use super::{tokenize_or_bail, Confidence, Pack, PatternRule, destructive_pattern, has_token};

/// Validator for `s3_recursive_delete`: matches `aws s3 rm --recursive` or
/// `aws s3 rb --force`.
fn s3_recursive_delete_validator(cmd: &str) -> bool {
    let tokens = tokenize_or_bail!(cmd, true);
    let has_rm_recursive = has_token(&tokens, "rm") && has_token(&tokens, "--recursive");
    let has_rb_force = has_token(&tokens, "rb") && has_token(&tokens, "--force");
    has_rm_recursive || has_rb_force
}

pub static AWS_PACK: Pack = Pack {
    id: "cloud.aws",
    name: "AWS",
    description: "Guards against destructive AWS operations",
    safe_patterns: &[],
    destructive_patterns: &[
        destructive_pattern!("terminate_instances", r"\baws\s+ec2\s+terminate-instances\b", "Permanently destroys EC2 instances", "Use `aws ec2 stop-instances` to stop without terminating", Confidence::High),
        destructive_pattern!("delete_db", r"\baws\s+rds\s+delete-db-(?:instance|cluster)\b", "Permanently deletes database instances", "Create a final snapshot first with `--final-db-snapshot-identifier`", Confidence::High),
        destructive_pattern!("s3_recursive_delete", r"\baws\s+s3\s+(?:rm|rb)\b", s3_recursive_delete_validator, "Recursively deletes S3 objects or force-removes buckets", "Use `aws s3 ls` to inspect first; delete specific prefixes", Confidence::High),
        destructive_pattern!("iam_delete", r"\baws\s+iam\s+delete-(?:user|role|policy)\b", "Removes IAM identities and their permissions", "Use `aws iam list-*` to review before deletion", Confidence::Medium),
    ],
};

pub static IAC_PACK: Pack = Pack {
    id: "cloud.iac",
    name: "Infrastructure as Code",
    description: "Guards against destructive IaC operations",
    safe_patterns: &[],
    destructive_patterns: &[
        destructive_pattern!("terraform_destroy", r"\bterraform\s+(?:destroy|apply\s+-destroy)\b", "Destroys all managed infrastructure resources", "Use `terraform plan -destroy` to preview what will be destroyed", Confidence::High),
        destructive_pattern!("pulumi_destroy", r"\bpulumi\s+destroy\b", "Destroys all managed infrastructure resources", "Use `pulumi preview --diff` to review changes first", Confidence::High),
    ],
};

/// Validator for `storage_delete`: matches `gsutil rm -r`/`--recursive` or
/// `gsutil rb -f`/`--force`.
fn gsutil_recursive_validator(cmd: &str) -> bool {
    let tokens = tokenize_or_bail!(cmd, true);
    let has_rm_recursive =
        has_token(&tokens, "rm") && (has_token(&tokens, "-r") || has_token(&tokens, "--recursive"));
    let has_rb_force =
        has_token(&tokens, "rb") && (has_token(&tokens, "-f") || has_token(&tokens, "--force"));
    has_rm_recursive || has_rb_force
}

pub static GCP_PACK: Pack = Pack {
    id: "cloud.gcp",
    name: "Google Cloud",
    description: "Guards against destructive GCP operations",
    safe_patterns: &[],
    destructive_patterns: &[
        destructive_pattern!("compute_delete", r"\bgcloud\s+compute\s+instances\s+delete\b", "Permanently destroys Compute Engine instances", "Use `gcloud compute instances stop` to stop without deleting", Confidence::High),
        destructive_pattern!("storage_delete", r"\bgsutil\s+(?:rm|rb)\b", gsutil_recursive_validator, "Recursively deletes Cloud Storage objects or force-removes buckets", "Use `gsutil ls` to inspect first; delete specific objects", Confidence::High),
        destructive_pattern!("sql_delete", r"\bgcloud\s+sql\s+instances\s+delete\b", "Permanently deletes Cloud SQL instances", "Create a backup first with `gcloud sql backups create`", Confidence::High),
    ],
};

pub static AZURE_PACK: Pack = Pack {
    id: "cloud.azure",
    name: "Azure",
    description: "Guards against destructive Azure operations",
    safe_patterns: &[],
    destructive_patterns: &[
        destructive_pattern!("vm_delete", r"\baz\s+vm\s+delete\b", "Permanently destroys virtual machines", "Use `az vm deallocate` to stop without deleting", Confidence::High),
        destructive_pattern!("group_delete", r"\baz\s+group\s+delete\b", "Deletes a resource group and all resources within it", "Use `az group show` to review contents first", Confidence::High),
        destructive_pattern!("storage_delete", r"\baz\s+storage\s+(?:blob\s+delete|container\s+delete)\b", "Deletes storage blobs or containers", "Use `az storage blob list` to review contents first", Confidence::Medium),
    ],
};

#[cfg(test)]
mod tests {
    use regex::Regex;

    use super::*;
    use super::super::tests::rule_by_id;

    #[test]
    fn terminate_instances_matches() {
        let re = Regex::new(rule_by_id(&AWS_PACK, "terminate_instances").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("aws ec2 terminate-instances --instance-ids i-123"));
        assert!(!re.is_match("aws ec2 stop-instances --instance-ids i-123"));
    }

    #[test]
    fn delete_db_matches() {
        let re = Regex::new(rule_by_id(&AWS_PACK, "delete_db").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("aws rds delete-db-instance --db-instance-identifier mydb"));
        assert!(re.is_match("aws rds delete-db-cluster --db-cluster-identifier mycluster"));
        assert!(!re.is_match("aws rds describe-db-instances"));
    }

    #[test]
    fn s3_recursive_delete_validator_cases() {
        assert!(s3_recursive_delete_validator(
            "aws s3 rm s3://bucket --recursive"
        ));
        assert!(s3_recursive_delete_validator(
            "aws s3 rb s3://bucket --force"
        ));
        assert!(!s3_recursive_delete_validator("aws s3 rm s3://bucket/file"));
        assert!(!s3_recursive_delete_validator("aws s3 rb s3://bucket"));
    }

    #[test]
    fn iam_delete_matches() {
        let re = Regex::new(rule_by_id(&AWS_PACK, "iam_delete").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("aws iam delete-user --user-name bob"));
        assert!(re.is_match("aws iam delete-role --role-name admin"));
        assert!(re.is_match("aws iam delete-policy --policy-arn arn:aws:..."));
        assert!(!re.is_match("aws iam list-users"));
    }

    #[test]
    fn terraform_destroy_matches() {
        let re = Regex::new(rule_by_id(&IAC_PACK, "terraform_destroy").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("terraform destroy"));
        assert!(re.is_match("terraform apply -destroy"));
        assert!(!re.is_match("terraform plan -destroy"));
        assert!(!re.is_match("terraform apply"));
    }

    #[test]
    fn pulumi_destroy_matches() {
        let re = Regex::new(rule_by_id(&IAC_PACK, "pulumi_destroy").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("pulumi destroy"));
        assert!(!re.is_match("pulumi preview"));
    }

    // GCP tests

    #[test]
    fn gcp_compute_delete_matches() {
        let re = Regex::new(rule_by_id(&GCP_PACK, "compute_delete").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("gcloud compute instances delete my-instance"));
        assert!(!re.is_match("gcloud compute instances stop my-instance"));
        assert!(!re.is_match("gcloud compute instances list"));
    }

    #[test]
    fn gsutil_recursive_validator_cases() {
        assert!(gsutil_recursive_validator("gsutil rm -r gs://bucket"));
        assert!(gsutil_recursive_validator(
            "gsutil rm --recursive gs://bucket"
        ));
        assert!(gsutil_recursive_validator("gsutil rb -f gs://bucket"));
        assert!(gsutil_recursive_validator("gsutil rb --force gs://bucket"));
        assert!(!gsutil_recursive_validator("gsutil rm gs://bucket/file"));
        assert!(!gsutil_recursive_validator("gsutil rb gs://bucket"));
    }

    #[test]
    fn gcp_sql_delete_matches() {
        let re = Regex::new(rule_by_id(&GCP_PACK, "sql_delete").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("gcloud sql instances delete mydb"));
        assert!(!re.is_match("gcloud sql instances describe mydb"));
    }

    // Azure tests

    #[test]
    fn azure_vm_delete_matches() {
        let re = Regex::new(rule_by_id(&AZURE_PACK, "vm_delete").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("az vm delete --name myvm --resource-group rg"));
        assert!(!re.is_match("az vm deallocate --name myvm"));
        assert!(!re.is_match("az vm list"));
    }

    #[test]
    fn azure_group_delete_matches() {
        let re = Regex::new(rule_by_id(&AZURE_PACK, "group_delete").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("az group delete --name my-rg"));
        assert!(!re.is_match("az group show --name my-rg"));
        assert!(!re.is_match("az group list"));
    }

    #[test]
    fn azure_storage_delete_matches() {
        let re = Regex::new(rule_by_id(&AZURE_PACK, "storage_delete").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("az storage blob delete --container mycontainer --name myblob"));
        assert!(re.is_match("az storage container delete --name mycontainer"));
        assert!(!re.is_match("az storage blob list --container mycontainer"));
        assert!(!re.is_match("az storage container list"));
    }
}
