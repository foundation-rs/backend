## Simple utility for push local content to server via ssh

> create .inventory.yaml in home directory

> build and run

> and will see instructions

## inventory file example

content: "@HOME/Desktop"
servers:
    sia:
        uri: 10.10.112.20
        user: acc-server-admin
        password: shwsa3nn
        description: experimental kubernetes server
        path-prefix: SIA/sia-system/configuration

    devel:
        uri: 10.10.112.21
        user: acc-server-admin
        password: shwsa3nn
        description: application server based on k8s and quarkus
        path-prefix: SIA-NEXT/sia-system/config-k8s

    registry:
        uri: 10.10.112.22
        user: acc-server-admin
        password: shwsa3nn
        description: docker registry server
        path-prefix: SIA-NEXT/sia-system/config-registry
