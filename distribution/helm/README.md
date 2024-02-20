# 🐻‍❄️💐 ume :: Helm Chart
This is the official [Helm](https://helm.sh) chart for Noel's [`ume`](https://floofy.dev/oss/ume) made in Rust.

## Requirements
- Kubernetes 1.26 or higher
- Helm 3.12 or higher

## Installation
```shell
$ helm repo add https://charts.noelware.org/~/noel
$ helm install ume noel/ume
```

## Parameters

### Global Parameters

Contains any global parameters that will affected all objects in the `ume` Helm chart.

| Name                              | Description                                                                                                                                                                                              | Value           |
| --------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------- |
| `global.replicas`                 | Amount of replicas to spawn                                                                                                                                                                              | `1`             |
| `global.resources`                | Resource list to apply to all containers.                                                                                                                                                                | `{}`            |
| `global.fullNameOverride`         | String to fully override the Helm installation name for all objects                                                                                                                                      | `""`            |
| `global.nameOverride`             | String to override the Helm installation name for all objects, will be in conjunction with a prefix of `<install-name>-`                                                                                 | `""`            |
| `global.clusterDomain`            | Domain host that maps to the cluster                                                                                                                                                                     | `cluster.local` |
| `global.nodeSelector`             | Selector labels to apply to contraint the pods to specific nodes. Read more in the [Kubernetes documentation](https://kubernetes.io/docs/concepts/scheduling-eviction/assign-pod-node/#nodeselector).    | `{}`            |
| `global.tolerations`              | List of all taints/tolerations to apply in conjunction with `global.affinity`. Read more in the [Kubernetes documentation](https://kubernetes.io/docs/concepts/scheduling-eviction/taint-and-toleration) | `[]`            |
| `global.affinity`                 | Map of all the affinity to apply to the spawned Pods. Read more in the [Kubernetes documentation](https://kubernetes.io/docs/tasks/configure-pod-container/assign-pods-nodes-using-node-affinity/).      | `{}`            |
| `global.annotations`              | Map of annotations to append to on all objects that this Helm chart creates.                                                                                                                             | `{}`            |
| `global.extraEnvVars`             | List of extra environment variables to append to all init/sidecar containers and normal containers.                                                                                                      | `[]`            |
| `global.initContainers`           | List of init containers to create.                                                                                                                                                                       | `[]`            |
| `global.podSecurityContext`       | Security context for all spawned Pods. Read more in the [Kubernetes documentation](https://kubernetes.io/docs/tasks/configure-pod-container/security-context/).                                          | `{}`            |
| `global.containerSecurityContext` | Security context for all init, sidecar, and normal containers. Read more in the [Kubernetes documentation](https://kubernetes.io/docs/tasks/configure-pod-container/security-context/).                  | `{}`            |

### Docker Image Parameters

Parameters to modify the Docker image that is ran.

| Name               | Description                                                                                                     | Value        |
| ------------------ | --------------------------------------------------------------------------------------------------------------- | ------------ |
| `image.pullPolicy` | [Pull policy](https://kubernetes.io/docs/concepts/containers/images/#image-pull-policy) when pulling the image. | `""`         |
| `image.registry`   | Registry URL to point to. For Docker Hub, use an empty string.                                                  | `""`         |
| `image.image`      | Image name.                                                                                                     | `auguwu/ume` |
| `image.tag`        | The tag of the image. Keep this as a empty string if you wish to use the default app's version.                 | `""`         |
| `image.digest`     | Digest in the form of `<alg>:<hex>`, this will replace the `image.tag` property if this is not empty.           | `""`         |

### Service Account Parameters

| Name                         | Description                                                                                | Value  |
| ---------------------------- | ------------------------------------------------------------------------------------------ | ------ |
| `serviceAccount.create`      | Whether or not if the service account should be created for this Helm installation.        | `true` |
| `serviceAccount.annotations` | Any additional annotations to append to this ServiceAccount                                | `{}`   |
| `serviceAccount.name`        | The name of the service account, this will be the Helm installation name if this is empty. | `""`   |
