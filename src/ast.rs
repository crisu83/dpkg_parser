/// Node kind enumerable.
#[derive(Debug)]
pub enum NodeKind {
    Document,
    Package,
    Library,
}

/// Describes a document node.
#[derive(Debug)]
pub struct Document {
    pub kind: NodeKind,
    pub packages: Vec<Package>,
}

impl Document {
    pub fn new(packages: Vec<Package>) -> Document {
        Document {
            kind: NodeKind::Document,
            packages,
        }
    }
}

/// Describes a package node.
#[derive(Debug)]
pub struct Package {
    pub kind: NodeKind,
    pub name: String,
    pub description: String,
    pub depends: Vec<Library>,
}

impl Package {
    pub fn new(name: String, description: String, depends: Vec<Library>) -> Package {
        Package {
            kind: NodeKind::Package,
            name,
            description,
            depends,
        }
    }
}

/// Describes a library node (e.g. a dependency).
#[derive(Debug)]
pub struct Library {
    pub kind: NodeKind,
    pub name: String,
    pub alternates: Vec<String>,
}

impl Library {
    pub fn new(name: String, alternates: Vec<String>) -> Library {
        Library {
            kind: NodeKind::Library,
            name,
            alternates,
        }
    }
}
