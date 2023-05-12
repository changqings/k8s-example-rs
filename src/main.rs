use k8s_openapi::api::core::v1::Pod;
use kube::api::{Api, ListParams, Patch, PatchParams, ResourceExt};
use kube::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read the environment to find config for kube client.
    // Note that this tries an in-cluster configuration first,
    // then falls back on a kubeconfig file.
    let client = Client::try_default().await?;

    // Interact with pods in the configured namespace with the typed interface from k8s-openapi
    let pods: Api<Pod> = Api::default_namespaced(client);

    // Create a Pod (cheating here with json, but it has to validate against the type):
    let patch: Pod = serde_json::from_value(serde_json::json!({
        "apiVersion": "v1",
        "kind": "Pod",
        "metadata": {
            "name": "my-pod"
        },
        "spec": {
            "containers": [
                {
                    "name": "my-container",
                    "image": "nginx:alpine",
                    "resources":
                      {
                        "limits": {
                            "cpu": "100m",
                            "memory": "128Mi"
                        }
                      },
                },
            ],
        }
    }))?;

    // Apply the Pod via server-side apply
    let params = PatchParams::apply("myapp");
    let result = pods.patch("my-pod", &params, &Patch::Apply(&patch)).await?;

    //
    println!(
        "pod {:?} have host_ip = {:?}",
        result.metadata.name,
        result.status.unwrap().host_ip
    );
    // List pods in the configured namespace
    for p in pods.list(&ListParams::default()).await? {
        println!(
            "found pod={},ip={}",
            p.name_any(),
            p.status
                .unwrap()
                .pod_ip
                .unwrap_or(String::from("pod not ready"))
        );
    }

    //

    Ok(())
}
