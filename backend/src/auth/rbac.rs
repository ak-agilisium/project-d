//! Role-based access control.
//!
//! Roles are sourced from the Cognito `cognito:groups` claim. Create these
//! groups in your user pool with names matching [`Role::as_group`]:
//!
//! | Role          | Cognito group  |
//! |---------------|----------------|
//! | Super Admin   | `super_admin`  |
//! | Admin         | `admin`        |
//! | Manager       | `manager`      |
//! | Doctor        | `doctor`       |
//! | Front Office  | `front_office` |
//!
//! A user may belong to several groups; their effective permissions are the
//! union across all of their roles. **Clinical** data is deliberately withheld
//! from non-clinical roles (including `admin`): only `super_admin` and `doctor`
//! may read or write it.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    SuperAdmin,
    Admin,
    Manager,
    Doctor,
    FrontOffice,
}

/// Fine-grained capabilities checked by handlers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    // Patient master
    PatientRead,
    PatientWrite,
    PatientDelete,
    // Reference data
    DoctorRead,
    DoctorWrite,
    DepartmentRead,
    DepartmentWrite,
    // Scheduling & OPD visits
    AppointmentRead,
    AppointmentWrite,
    AppointmentDelete,
    VisitRead,
    VisitWrite,
    // Clinical (restricted)
    ClinicalRead,
    ClinicalWrite,
    PrescriptionRead,
    PrescriptionWrite,
    LabRead,
    LabWrite,
    // Billing
    BillingRead,
    BillingWrite,
    InsuranceRead,
    InsuranceWrite,
    // Files
    ReportRead,
    ReportWrite,
    ReportDelete,
    DocumentRead,
    DocumentWrite,
    // Administration
    AuditRead,
    UserManage,
}

impl Role {
    /// Parse a Cognito group name into a role. Accepts a few common spellings.
    pub fn from_group(group: &str) -> Option<Role> {
        match group
            .trim()
            .to_ascii_lowercase()
            .replace([' ', '-'], "_")
            .as_str()
        {
            "super_admin" | "superadmin" => Some(Role::SuperAdmin),
            "admin" | "administrator" => Some(Role::Admin),
            "manager" => Some(Role::Manager),
            "doctor" | "physician" => Some(Role::Doctor),
            "front_office" | "frontoffice" | "reception" | "receptionist" => {
                Some(Role::FrontOffice)
            }
            _ => None,
        }
    }

    /// The canonical Cognito group name for this role.
    pub fn as_group(self) -> &'static str {
        match self {
            Role::SuperAdmin => "super_admin",
            Role::Admin => "admin",
            Role::Manager => "manager",
            Role::Doctor => "doctor",
            Role::FrontOffice => "front_office",
        }
    }

    /// Whether this role grants the given permission.
    pub fn grants(self, perm: Permission) -> bool {
        use Permission::*;
        match self {
            // Everything.
            Role::SuperAdmin => true,

            // Full access EXCEPT clinical information (privacy).
            Role::Admin => !matches!(perm, ClinicalRead | ClinicalWrite),

            // Oversight & finance: broad read, billing write, no clinical, no deletes.
            Role::Manager => matches!(
                perm,
                PatientRead
                    | DoctorRead
                    | DepartmentRead
                    | AppointmentRead
                    | VisitRead
                    | BillingRead
                    | BillingWrite
                    | InsuranceRead
                    | ReportRead
                    | ReportWrite
                    | DocumentRead
                    | PrescriptionRead
                    | LabRead
                    | AuditRead
            ),

            // Clinical: visits, clinical records, prescriptions, lab orders.
            Role::Doctor => matches!(
                perm,
                PatientRead
                    | DoctorRead
                    | DepartmentRead
                    | AppointmentRead
                    | AppointmentWrite
                    | VisitRead
                    | VisitWrite
                    | ClinicalRead
                    | ClinicalWrite
                    | PrescriptionRead
                    | PrescriptionWrite
                    | LabRead
                    | LabWrite
                    | BillingRead
                    | ReportRead
                    | ReportWrite
                    | DocumentRead
                    | DocumentWrite
            ),

            // Reception: registration, visits, scheduling, billing, insurance, docs.
            Role::FrontOffice => matches!(
                perm,
                PatientRead
                    | PatientWrite
                    | DoctorRead
                    | DepartmentRead
                    | AppointmentRead
                    | AppointmentWrite
                    | VisitRead
                    | VisitWrite
                    | BillingRead
                    | BillingWrite
                    | InsuranceRead
                    | InsuranceWrite
                    | ReportRead
                    | ReportWrite
                    | DocumentRead
                    | DocumentWrite
            ),
        }
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_group())
    }
}
