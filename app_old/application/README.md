# UMLCAD Application Hosts

This directory contains application hosts and demonstrations that consume the Application Framework.

Application hosts are not framework libraries.

The intended flow is:

\`\`\`
Demo / Application Host
        |
        v
UMLCAD.Framework
        |
        v
Application Layer libraries
        |
        v
UMLCAD.Kernel
        |
        v
Mathematical kernel
\`\`\`

No host or demo may introduce a second kernel-access path.
