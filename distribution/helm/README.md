# 🐻‍❄️💐 `ume` | Helm Chart
Official Helm chart of Noel's image service, `ume`.

## Requirements
- Kubernetes 1.29+
- Helm 3.15+

## Installation
```shell
# With the `charted` plugin
$ helm install ume charted://noel/ume

# Without the `charted` plugin
$ helm repo add noel https://charts.noelware.org/~/noel
$ helm install ume noel/ume
```

## Parameters

[k8s-resources]:
[k8s-node-selector]:
[k8s-taint-and-toleration]: https://kubernetes.io/docs/concepts/scheduling-eviction/taint-and-toleration
[k8s-node-affinity]: https://kubernetes.io/docs/tasks/configure-pod-container/assign-pods-nodes-using-node-affinity/
[k8s-security-context]: https://kubernetes.io/docs/tasks/configure-pod-container/security-context/
[k8s-pod-dns-config]: https://kubernetes.io/docs/concepts/services-networking/dns-pod-service/#pod-dns-config
[k8s-image-pull-policy]: https://kubernetes.io/docs/concepts/containers/images/#image-pull-policy
[k8s-pull-private-images]: https://kubernetes.io/docs/tasks/configure-pod-container/pull-image-private-registry/
