# UMLCAD V5 — Authentication and Authorization Boundary

## 1. Purpose

Authentication and authorization are surrounding application concerns. They are not part of the mathematical CAD kernel.

For GitHub-hosted UMLCAD resources, GitHub remains the identity, repository, and file-access authority. The browser is the user-facing authentication/authorization surface.

The V5 kernel must not invent a second repository permission system.

## 2. Boundary

```text
USER
  ↓
BROWSER
  ↓ GitHub authentication / authorization
AUTHORIZED APPLICATION CONTEXT
  ↓
HOST / SERVER
  ↓
V5 KERNEL
```

The native CAD client is an interaction/rendering endpoint, not an authorization authority.

## 3. Kernel rule

The kernel accepts an already-authorized application context at its host boundary. Kernel operations themselves do not implement GitHub OAuth, passwords, roles, or repository ACLs.

A valid CAD request is not automatically an authorized request. The host/session layer must establish authorization before invoking the kernel.

## 4. What must not be duplicated

Do not add merely for V5:

- desktop username/password accounts;
- a local GitHub credential authority;
- a parallel repository ACL database;
- client-side role flags as authorization;
- a second permission model for the same GitHub repository.

## 5. Security rule

Client capability data is descriptive, not authorization.

```text
client says operation is possible
        ≠
server says operation is authorized
```

The host must validate both authorization and the semantic operation before committing a kernel mutation.

## 6. AI rule

AI-generated authentication code must preserve this separation:

```text
GitHub identity/access
        ↓
browser authorization flow
        ↓
authorized application/session context
        ↓
host/security validation
        ↓
V5 semantic kernel
```

If future requirements need UMLCAD-specific entitlements, those must be introduced as an explicit architecture rather than silently mixing them into GitHub authorization semantics.
