load("@gazelle//:deps.bzl", "go_repository")

def go_dependencies():
    go_repository(
        name = "com_github_aymanbagabas_go_udiff",
        importpath = "github.com/aymanbagabas/go-udiff",
        sum = "h1:OEIrQ8maEeDBXQDoGCbbTTXYJMYRCRO1fnodZ12Gv5o=",
        version = "v0.4.1",
    )
    go_repository(
        name = "com_github_bazel_contrib_bazel_gazelle_v2",
        importpath = "github.com/bazel-contrib/bazel-gazelle/v2",
        sum = "h1:lSINS7DK4xjm0Z4CBd7onO63uIe6WUsEZ/0voExa+JY=",
        version = "v2.0.0-2",
    )
    go_repository(
        name = "com_github_bazelbuild_bazel_gazelle",
        importpath = "github.com/bazelbuild/bazel-gazelle",
        sum = "h1:WtLCDPJTj06AWZizuN0SpkX8njjH1meOFgYbia6mD64=",
        version = "v0.51.3",
    )
    go_repository(
        name = "com_github_bazelbuild_buildtools",
        importpath = "github.com/bazelbuild/buildtools",
        sum = "h1:njQAmjTv/YHRm/0Lfv9DXHFZ4MdT2IA/RKHTnqZkgDw=",
        version = "v0.0.0-20250930140053-2eb4fccefb52",
    )
    go_repository(
        name = "com_github_bazelbuild_rules_go",
        importpath = "github.com/bazelbuild/rules_go",
        sum = "h1:F7jqdw0Zp/RtWFDbLFuVa2DaPjUQkqZOLCns8vCxSWg=",
        version = "v0.61.1",
    )
    go_repository(
        name = "com_github_bmatcuk_doublestar_v4",
        importpath = "github.com/bmatcuk/doublestar/v4",
        sum = "h1:X8jg9rRZmJd4yRy7ZeNDRnM+T3ZfHv15JiBJ/avrEXE=",
        version = "v4.9.1",
    )
    go_repository(
        name = "com_github_cespare_xxhash_v2",
        importpath = "github.com/cespare/xxhash/v2",
        sum = "h1:UL815xU9SqsFlibzuggzjXhog7bL6oX9BbNZnL2UFvs=",
        version = "v2.3.0",
    )
    go_repository(
        name = "com_github_cncf_xds_go",
        importpath = "github.com/cncf/xds/go",
        sum = "h1:boJj011Hh+874zpIySeApCX4GeOjPl9qhRF3QuIZq+Q=",
        version = "v0.0.0-20241223141626-cff3c89139a3",
    )
    go_repository(
        name = "com_github_envoyproxy_go_control_plane",
        importpath = "github.com/envoyproxy/go-control-plane",
        sum = "h1:zEqyPVyku6IvWCFwux4x9RxkLOMUL+1vC9xUFv5l2/M=",
        version = "v0.13.4",
    )
    go_repository(
        name = "com_github_envoyproxy_go_control_plane_envoy",
        importpath = "github.com/envoyproxy/go-control-plane/envoy",
        sum = "h1:jb83lalDRZSpPWW2Z7Mck/8kXZ5CQAFYVjQcdVIr83A=",
        version = "v1.32.4",
    )
    go_repository(
        name = "com_github_envoyproxy_go_control_plane_ratelimit",
        importpath = "github.com/envoyproxy/go-control-plane/ratelimit",
        sum = "h1:/G9QYbddjL25KvtKTv3an9lx6VBE2cnb8wp1vEGNYGI=",
        version = "v0.1.0",
    )
    go_repository(
        name = "com_github_envoyproxy_protoc_gen_validate",
        importpath = "github.com/envoyproxy/protoc-gen-validate",
        sum = "h1:DEo3O99U8j4hBFwbJfrz9VtgcDfUKS7KJ7spH3d86P8=",
        version = "v1.2.1",
    )
    go_repository(
        name = "com_github_fsnotify_fsnotify",
        importpath = "github.com/fsnotify/fsnotify",
        sum = "h1:8JEhPFa5W2WU7YfeZzPNqzMP6Lwt7L2715Ggo0nosvA=",
        version = "v1.7.0",
    )
    go_repository(
        name = "com_github_go_logr_logr",
        importpath = "github.com/go-logr/logr",
        sum = "h1:6pFjapn8bFcIbiKo3XT4j/BhANplGihG6tvd+8rYgrY=",
        version = "v1.4.2",
    )
    go_repository(
        name = "com_github_go_logr_stdr",
        importpath = "github.com/go-logr/stdr",
        sum = "h1:hSWxHoqTgW2S2qGc0LTAI563KZ5YKYRhT3MFKZMbjag=",
        version = "v1.2.2",
    )
    go_repository(
        name = "com_github_gogo_protobuf",
        importpath = "github.com/gogo/protobuf",
        sum = "h1:Ov1cvc58UF3b5XjBnZv7+opcTcQFZebYjWzi34vdm4Q=",
        version = "v1.3.2",
    )
    go_repository(
        name = "com_github_golang_glog",
        importpath = "github.com/golang/glog",
        sum = "h1:CNNw5U8lSiiBk7druxtSHHTsRWcxKoac6kZKm2peBBc=",
        version = "v1.2.4",
    )
    go_repository(
        name = "com_github_golang_mock",
        importpath = "github.com/golang/mock",
        sum = "h1:YojYx61/OLFsiv6Rw1Z96LpldJIy31o+UHmwAUMJ6/U=",
        version = "v1.7.0-rc.1",
    )
    go_repository(
        name = "com_github_golang_protobuf",
        importpath = "github.com/golang/protobuf",
        sum = "h1:i7eJL8qZTpSEXOPTxNKhASYpMn+8e5Q6AdndVa1dWek=",
        version = "v1.5.4",
    )
    go_repository(
        name = "com_github_google_go_cmp",
        importpath = "github.com/google/go-cmp",
        sum = "h1:wk8382ETsv4JYUZwIsn6YpYiWiBsYLSJiTsyBybVuN8=",
        version = "v0.7.0",
    )
    go_repository(
        name = "com_github_google_uuid",
        importpath = "github.com/google/uuid",
        sum = "h1:NIvaJDMOsjHA8n1jAhLSgzrAzy1Hgr+hNrb57e+94F0=",
        version = "v1.6.0",
    )
    go_repository(
        name = "com_github_googlecloudplatform_opentelemetry_operations_go_detectors_gcp",
        importpath = "github.com/GoogleCloudPlatform/opentelemetry-operations-go/detectors/gcp",
        sum = "h1:3c8yed4lgqTt+oTQ+JNMDo+F4xprBf+O/il4ZC0nRLw=",
        version = "v1.25.0",
    )
    go_repository(
        name = "com_github_kisielk_errcheck",
        importpath = "github.com/kisielk/errcheck",
        sum = "h1:e8esj/e4R+SAOwFwN+n3zr0nYeCyeweozKfO23MvHzY=",
        version = "v1.5.0",
    )
    go_repository(
        name = "com_github_kisielk_gotool",
        importpath = "github.com/kisielk/gotool",
        sum = "h1:AV2c/EiW3KqPNT9ZKl07ehoAGi4C5/01Cfbblndcapg=",
        version = "v1.0.0",
    )
    go_repository(
        name = "com_github_planetscale_vtprotobuf",
        importpath = "github.com/planetscale/vtprotobuf",
        sum = "h1:GFCKgmp0tecUJ0sJuv4pzYCqS9+RGSn52M3FUwPs+uo=",
        version = "v0.6.1-0.20240319094008-0393e58bdf10",
    )
    go_repository(
        name = "com_github_pmezard_go_difflib",
        importpath = "github.com/pmezard/go-difflib",
        sum = "h1:4DBwDE0NGyQoBHbLQYPwSUPoCMWR5BEzIk/f1lZbAQM=",
        version = "v1.0.0",
    )
    go_repository(
        name = "com_github_yuin_goldmark",
        importpath = "github.com/yuin/goldmark",
        sum = "h1:fVcFKWvrslecOb/tg+Cc05dkeYx540o0FuFt3nUVDoE=",
        version = "v1.4.13",
    )
    go_repository(
        name = "com_google_cloud_go",
        importpath = "cloud.google.com/go",
        sum = "h1:tvZe1mgqRxpiVa3XlIGMiPcEUbP1gNXELgD4y/IXmeQ=",
        version = "v0.118.0",
    )
    go_repository(
        name = "com_google_cloud_go_accessapproval",
        importpath = "cloud.google.com/go/accessapproval",
        sum = "h1:axlU03FRiXDNupsmPG7LKzuS4Enk1gf598M62lWVB74=",
        version = "v1.8.3",
    )
    go_repository(
        name = "com_google_cloud_go_accesscontextmanager",
        importpath = "cloud.google.com/go/accesscontextmanager",
        sum = "h1:8zVoeiBa4erMCLEXltOcqVEsZhS26JZ5/Vrgs59eQiI=",
        version = "v1.9.3",
    )
    go_repository(
        name = "com_google_cloud_go_aiplatform",
        importpath = "cloud.google.com/go/aiplatform",
        sum = "h1:vnqsPkgcwlDEpWl9t6C3/HLfHeweuGXs2gcYTzH6dMs=",
        version = "v1.70.0",
    )
    go_repository(
        name = "com_google_cloud_go_analytics",
        importpath = "cloud.google.com/go/analytics",
        sum = "h1:hX6JAsNbXd2uVjqjIuMcKpmhIybKrEunBiGxK4SwEFI=",
        version = "v0.25.3",
    )
    go_repository(
        name = "com_google_cloud_go_apigateway",
        importpath = "cloud.google.com/go/apigateway",
        sum = "h1:Mn7cC5iWJz+cSMS/Hb+N2410CpZ6c8XpJKaexBl0Gxs=",
        version = "v1.7.3",
    )
    go_repository(
        name = "com_google_cloud_go_apigeeconnect",
        importpath = "cloud.google.com/go/apigeeconnect",
        sum = "h1:Wlr+30Tha0SMCvQYZKdrh+HkpOyl0CQFSlzeY/Gg1gs=",
        version = "v1.7.3",
    )
    go_repository(
        name = "com_google_cloud_go_apigeeregistry",
        importpath = "cloud.google.com/go/apigeeregistry",
        sum = "h1:j9CJg/oC884OX5cDpiwNt1ZlDXNV6Zb9Mp1YmRrOG0k=",
        version = "v0.9.3",
    )
    go_repository(
        name = "com_google_cloud_go_appengine",
        importpath = "cloud.google.com/go/appengine",
        sum = "h1:jrcanSzj9J1erevZuxldvsDwY+0k/DeFFzlnSfPGfL8=",
        version = "v1.9.3",
    )
    go_repository(
        name = "com_google_cloud_go_area120",
        importpath = "cloud.google.com/go/area120",
        sum = "h1:dPQ07rW4eku8OgNWDOaQaVGcE4+XfhH8BSbVwdVQ+wU=",
        version = "v0.9.3",
    )
    go_repository(
        name = "com_google_cloud_go_artifactregistry",
        importpath = "cloud.google.com/go/artifactregistry",
        sum = "h1:ZNXGB6+T7VmWdf6//VqxLdZ/sk0no8W0ujanHeJwDRw=",
        version = "v1.16.1",
    )
    go_repository(
        name = "com_google_cloud_go_asset",
        importpath = "cloud.google.com/go/asset",
        sum = "h1:6oNgjcs5KCPGBD71G0IccK6TfeFsEtBTyQ3Q+Dn09bs=",
        version = "v1.20.4",
    )
    go_repository(
        name = "com_google_cloud_go_assuredworkloads",
        importpath = "cloud.google.com/go/assuredworkloads",
        sum = "h1:RU1WhF1zMggdXAZ+ezYTn4Eh/FdiX7sz8lLXGERn4Po=",
        version = "v1.12.3",
    )
    go_repository(
        name = "com_google_cloud_go_automl",
        importpath = "cloud.google.com/go/automl",
        sum = "h1:vkD+hQ75SMINMgJBT/KDpFYvfQLzJbtIQZdw0AWq8Rs=",
        version = "v1.14.4",
    )
    go_repository(
        name = "com_google_cloud_go_baremetalsolution",
        importpath = "cloud.google.com/go/baremetalsolution",
        sum = "h1:OL+KT+wCumdDhG44aeqGAdkwdT8Wa4Lh+o4INM+CQjw=",
        version = "v1.3.3",
    )
    go_repository(
        name = "com_google_cloud_go_batch",
        importpath = "cloud.google.com/go/batch",
        sum = "h1:TLfFZJXu+89CGbDK2mMql8f6HHFXarr8uUsaQ6wKatU=",
        version = "v1.11.5",
    )
    go_repository(
        name = "com_google_cloud_go_beyondcorp",
        importpath = "cloud.google.com/go/beyondcorp",
        sum = "h1:ezavJc0Gzh4N8zBskO/DnUVMWPa8lqH/tmQSyaknmCA=",
        version = "v1.1.3",
    )
    go_repository(
        name = "com_google_cloud_go_bigquery",
        importpath = "cloud.google.com/go/bigquery",
        sum = "h1:ZZ1EOJMHTYf6R9lhxIXZJic1qBD4/x9loBIS+82moUs=",
        version = "v1.65.0",
    )
    go_repository(
        name = "com_google_cloud_go_bigtable",
        importpath = "cloud.google.com/go/bigtable",
        sum = "h1:eIgi3QLcN4aq8p6n9U/zPgmHeBP34sm9FiKq4ik/ZoY=",
        version = "v1.34.0",
    )
    go_repository(
        name = "com_google_cloud_go_billing",
        importpath = "cloud.google.com/go/billing",
        sum = "h1:xMlO3hc5BI0s23tRB40bL40xSpxUR1x3E07Y5/VWcjU=",
        version = "v1.20.1",
    )
    go_repository(
        name = "com_google_cloud_go_binaryauthorization",
        importpath = "cloud.google.com/go/binaryauthorization",
        sum = "h1:X8JRfmk0/vyRqLusEyAPr0nZCK6RKae9omB4lrit0XI=",
        version = "v1.9.3",
    )
    go_repository(
        name = "com_google_cloud_go_certificatemanager",
        importpath = "cloud.google.com/go/certificatemanager",
        sum = "h1:2UP31fg7b+y3F0OmNbPHOKPEJ+6LOMfxAXX4p8xGCy4=",
        version = "v1.9.3",
    )
    go_repository(
        name = "com_google_cloud_go_channel",
        importpath = "cloud.google.com/go/channel",
        sum = "h1:oHyO3QAZ6kdf6SwqnUTBz50ND6Nk2rxZtboUiF4dgLE=",
        version = "v1.19.2",
    )
    go_repository(
        name = "com_google_cloud_go_cloudbuild",
        importpath = "cloud.google.com/go/cloudbuild",
        sum = "h1:fYsJweKNT1b9cCQHoE3499n1Olr+7z50Ep7jnA+szDs=",
        version = "v1.19.2",
    )
    go_repository(
        name = "com_google_cloud_go_clouddms",
        importpath = "cloud.google.com/go/clouddms",
        sum = "h1:T/rkkKE0KhQFMcO3+QWL82xakA9kRumLXY1lq5adIts=",
        version = "v1.8.3",
    )
    go_repository(
        name = "com_google_cloud_go_cloudtasks",
        importpath = "cloud.google.com/go/cloudtasks",
        sum = "h1:rXdznKjCa7WpzmvR2plrn2KJ+RZC1oYxPiRWNQjjf3k=",
        version = "v1.13.3",
    )
    go_repository(
        name = "com_google_cloud_go_compute",
        importpath = "cloud.google.com/go/compute",
        sum = "h1:SObuy8Fs6woazArpXp1fsHCw+ZH4iJ/8dGGTxUhHZQA=",
        version = "v1.31.1",
    )
    go_repository(
        name = "com_google_cloud_go_compute_metadata",
        importpath = "cloud.google.com/go/compute/metadata",
        sum = "h1:A6hENjEsCDtC1k8byVsgwvVcioamEHvZ4j01OwKxG9I=",
        version = "v0.6.0",
    )
    go_repository(
        name = "com_google_cloud_go_contactcenterinsights",
        importpath = "cloud.google.com/go/contactcenterinsights",
        sum = "h1:xJoZbX0HM1zht8KxAB38hs2v4Hcl+vXGLo454LrdwxA=",
        version = "v1.17.1",
    )
    go_repository(
        name = "com_google_cloud_go_container",
        importpath = "cloud.google.com/go/container",
        sum = "h1:eaMrgOl6NCk+Blhh29GgUVe3QGo7IiJQlP0w/EwLoV0=",
        version = "v1.42.1",
    )
    go_repository(
        name = "com_google_cloud_go_containeranalysis",
        importpath = "cloud.google.com/go/containeranalysis",
        sum = "h1:1D8U75BeotZxrG4jR6NYBtOt+uAeBsWhpBZmSYLakQw=",
        version = "v0.13.3",
    )
    go_repository(
        name = "com_google_cloud_go_datacatalog",
        importpath = "cloud.google.com/go/datacatalog",
        sum = "h1:3bAfstDB6rlHyK0TvqxEwaeOvoN9UgCs2bn03+VXmss=",
        version = "v1.24.3",
    )
    go_repository(
        name = "com_google_cloud_go_dataflow",
        importpath = "cloud.google.com/go/dataflow",
        sum = "h1:+7IfIXzYWSybIIDGK9FN2uqBsP/5b/Y0pBYzNhcmKSU=",
        version = "v0.10.3",
    )
    go_repository(
        name = "com_google_cloud_go_dataform",
        importpath = "cloud.google.com/go/dataform",
        sum = "h1:ZpGkZV8OyhUhvN/tfLffU2ki5ERTtqOunkIaiVAhmw0=",
        version = "v0.10.3",
    )
    go_repository(
        name = "com_google_cloud_go_datafusion",
        importpath = "cloud.google.com/go/datafusion",
        sum = "h1:FTMtsf2nfGGlDCuE84/RvVaCcTIYE7WQSB0noeO0cwI=",
        version = "v1.8.3",
    )
    go_repository(
        name = "com_google_cloud_go_datalabeling",
        importpath = "cloud.google.com/go/datalabeling",
        sum = "h1:PqoA3gnOWaLcHCnqoZe4jh3jmiv6+Z7W2xUUkw/j4jE=",
        version = "v0.9.3",
    )
    go_repository(
        name = "com_google_cloud_go_dataplex",
        importpath = "cloud.google.com/go/dataplex",
        sum = "h1:oswf105Cr2EwHrW2n7wk3nRZQf7hCe3apE/GqJ8yjvY=",
        version = "v1.21.0",
    )
    go_repository(
        name = "com_google_cloud_go_dataproc_v2",
        importpath = "cloud.google.com/go/dataproc/v2",
        sum = "h1:2vOv471LrcSn91VNzijcH+OkDRLa3kdyymOfKqbwZ4c=",
        version = "v2.10.1",
    )
    go_repository(
        name = "com_google_cloud_go_dataqna",
        importpath = "cloud.google.com/go/dataqna",
        sum = "h1:lGUj2FYs650EUPDMV6plWBAoh8qH9Bu1KCz1PUYF2VY=",
        version = "v0.9.3",
    )
    go_repository(
        name = "com_google_cloud_go_datastore",
        importpath = "cloud.google.com/go/datastore",
        sum = "h1:NNpXoyEqIJmZFc0ACcwBEaXnmscUpcG4NkKnbCePmiM=",
        version = "v1.20.0",
    )
    go_repository(
        name = "com_google_cloud_go_datastream",
        importpath = "cloud.google.com/go/datastream",
        sum = "h1:j5cIRYJHjx/058aHa4Slip7fl62UTGHCJc4GL9bxQLQ=",
        version = "v1.12.1",
    )
    go_repository(
        name = "com_google_cloud_go_deploy",
        importpath = "cloud.google.com/go/deploy",
        sum = "h1:Hm3pXBzMFJFPOdwtDkg5e/LP53bXqIpwQpjwsVasjhU=",
        version = "v1.26.1",
    )
    go_repository(
        name = "com_google_cloud_go_dialogflow",
        importpath = "cloud.google.com/go/dialogflow",
        sum = "h1:6fU4IKLpvgpXqiUCE8gUp8eV5u629SCtiyXMudXtZSg=",
        version = "v1.64.1",
    )
    go_repository(
        name = "com_google_cloud_go_dlp",
        importpath = "cloud.google.com/go/dlp",
        sum = "h1:qAEGTTtC97zuDm6YPBozNvy4BLBszVCJah3efNytl3g=",
        version = "v1.20.1",
    )
    go_repository(
        name = "com_google_cloud_go_documentai",
        importpath = "cloud.google.com/go/documentai",
        sum = "h1:52RfiUsoblXcE57CfKJGnITWLxRM30BcqNk/BKZl2LI=",
        version = "v1.35.1",
    )
    go_repository(
        name = "com_google_cloud_go_domains",
        importpath = "cloud.google.com/go/domains",
        sum = "h1:wnqN5YwMrtLSjn+HB2sChgmZ6iocOta4Q41giQsiRjY=",
        version = "v0.10.3",
    )
    go_repository(
        name = "com_google_cloud_go_edgecontainer",
        importpath = "cloud.google.com/go/edgecontainer",
        sum = "h1:SwQuHQiheVfL7b5ar/AXDberiaqr/yiue8X55AdWnZU=",
        version = "v1.4.1",
    )
    go_repository(
        name = "com_google_cloud_go_errorreporting",
        importpath = "cloud.google.com/go/errorreporting",
        sum = "h1:isaoPwWX8kbAOea4qahcmttoS79+gQhvKsfg5L5AgH8=",
        version = "v0.3.2",
    )
    go_repository(
        name = "com_google_cloud_go_essentialcontacts",
        importpath = "cloud.google.com/go/essentialcontacts",
        sum = "h1:Paw495vxVyKuAgcQ2NQk09iRZBhPYRytknydEnvzcv4=",
        version = "v1.7.3",
    )
    go_repository(
        name = "com_google_cloud_go_eventarc",
        importpath = "cloud.google.com/go/eventarc",
        sum = "h1:RMymT7R87LaxKugOKwooOoheWXUm1NMeOfh3CVU9g54=",
        version = "v1.15.1",
    )
    go_repository(
        name = "com_google_cloud_go_filestore",
        importpath = "cloud.google.com/go/filestore",
        sum = "h1:vTXQI5qYKZ8dmCyHN+zVfaMyXCYbyZNM0CkPzpPUn7Q=",
        version = "v1.9.3",
    )
    go_repository(
        name = "com_google_cloud_go_firestore",
        importpath = "cloud.google.com/go/firestore",
        sum = "h1:cuydCaLS7Vl2SatAeivXyhbhDEIR8BDmtn4egDhIn2s=",
        version = "v1.18.0",
    )
    go_repository(
        name = "com_google_cloud_go_functions",
        importpath = "cloud.google.com/go/functions",
        sum = "h1:V0vCHSgFTUqKn57+PUXp1UfQY0/aMkveAw7wXeM3Lq0=",
        version = "v1.19.3",
    )
    go_repository(
        name = "com_google_cloud_go_gkebackup",
        importpath = "cloud.google.com/go/gkebackup",
        sum = "h1:djdExe/QgoKdp1gnIO1G5BoO1o/yGQOQJJEZ4QKTEXQ=",
        version = "v1.6.3",
    )
    go_repository(
        name = "com_google_cloud_go_gkeconnect",
        importpath = "cloud.google.com/go/gkeconnect",
        sum = "h1:YVpR0vlHSP/wD74PXEbKua4Aamud+wiYm4TiewNjD3M=",
        version = "v0.12.1",
    )
    go_repository(
        name = "com_google_cloud_go_gkehub",
        importpath = "cloud.google.com/go/gkehub",
        sum = "h1:yZ6lNJ9rNIoQmWrG14dB3+BFjS/EIRBf7Bo6jc5QWlE=",
        version = "v0.15.3",
    )
    go_repository(
        name = "com_google_cloud_go_gkemulticloud",
        importpath = "cloud.google.com/go/gkemulticloud",
        sum = "h1:8Z1rWFbnNGgB3KMFbg8OCiIiw2Hl1nxZWwZGU740XRs=",
        version = "v1.5.0",
    )
    go_repository(
        name = "com_google_cloud_go_gsuiteaddons",
        importpath = "cloud.google.com/go/gsuiteaddons",
        sum = "h1:QafYhVhyFGpidBUUlVhy6lUHFogFOycVYm9DV7MinhA=",
        version = "v1.7.3",
    )
    go_repository(
        name = "com_google_cloud_go_iam",
        importpath = "cloud.google.com/go/iam",
        sum = "h1:KFf8SaT71yYq+sQtRISn90Gyhyf4X8RGgeAVC8XGf3E=",
        version = "v1.3.1",
    )
    go_repository(
        name = "com_google_cloud_go_iap",
        importpath = "cloud.google.com/go/iap",
        sum = "h1:OWNYFHPyIBNHEAEFdVKOltYWe0g3izSrpFJW6Iidovk=",
        version = "v1.10.3",
    )
    go_repository(
        name = "com_google_cloud_go_ids",
        importpath = "cloud.google.com/go/ids",
        sum = "h1:wbFF7twu0XScFr+dtsVxTTttbFIRYt/SJjZiHFidtYE=",
        version = "v1.5.3",
    )
    go_repository(
        name = "com_google_cloud_go_iot",
        importpath = "cloud.google.com/go/iot",
        sum = "h1:aPWYQ+A1NX6ou/5U0nFAiXWdVT8OBxZYVZt2fBl2gWA=",
        version = "v1.8.3",
    )
    go_repository(
        name = "com_google_cloud_go_kms",
        importpath = "cloud.google.com/go/kms",
        sum = "h1:aQQ8esAIVZ1atdJRxihhdxGQ64/zEbJoJnCz/ydSmKg=",
        version = "v1.20.5",
    )
    go_repository(
        name = "com_google_cloud_go_language",
        importpath = "cloud.google.com/go/language",
        sum = "h1:8hmFMiS3wjjj3TX/U1zZYTgzwZoUjDbo9PaqcYEmuB4=",
        version = "v1.14.3",
    )
    go_repository(
        name = "com_google_cloud_go_lifesciences",
        importpath = "cloud.google.com/go/lifesciences",
        sum = "h1:Z05C+Ui953f0EQx9hJ1la6+QQl8ADrIs3iNwP5Elkpg=",
        version = "v0.10.3",
    )
    go_repository(
        name = "com_google_cloud_go_logging",
        importpath = "cloud.google.com/go/logging",
        sum = "h1:7j0HgAp0B94o1YRDqiqm26w4q1rDMH7XNRU34lJXHYc=",
        version = "v1.13.0",
    )
    go_repository(
        name = "com_google_cloud_go_longrunning",
        importpath = "cloud.google.com/go/longrunning",
        sum = "h1:3tyw9rO3E2XVXzSApn1gyEEnH2K9SynNQjMlBi3uHLg=",
        version = "v0.6.4",
    )
    go_repository(
        name = "com_google_cloud_go_managedidentities",
        importpath = "cloud.google.com/go/managedidentities",
        sum = "h1:b9xGs24BIjfyvLgCtJoClOZpPi8d8owPgWe5JEINgaY=",
        version = "v1.7.3",
    )
    go_repository(
        name = "com_google_cloud_go_maps",
        importpath = "cloud.google.com/go/maps",
        sum = "h1:u7U/DieTxYYMDyvHQ00la5ayXLjDImTfnhdAsyPZXyY=",
        version = "v1.17.1",
    )
    go_repository(
        name = "com_google_cloud_go_mediatranslation",
        importpath = "cloud.google.com/go/mediatranslation",
        sum = "h1:nRBjeaMLipw05Br+qDAlSCcCQAAlat4mvpafztbEVgc=",
        version = "v0.9.3",
    )
    go_repository(
        name = "com_google_cloud_go_memcache",
        importpath = "cloud.google.com/go/memcache",
        sum = "h1:XH/qT3GbbSH//R0JTqR77lRpBxaa0N9sHgAzfwbTrv0=",
        version = "v1.11.3",
    )
    go_repository(
        name = "com_google_cloud_go_metastore",
        importpath = "cloud.google.com/go/metastore",
        sum = "h1:jDqeCw6NGDRAPT9+2Y/EjnWAB0BfCcUfmPLOyhB0eHs=",
        version = "v1.14.3",
    )
    go_repository(
        name = "com_google_cloud_go_monitoring",
        importpath = "cloud.google.com/go/monitoring",
        sum = "h1:KQbnAC4IAH+5x3iWuPZT5iN9VXqKMzzOgqcYB6fqPDE=",
        version = "v1.22.1",
    )
    go_repository(
        name = "com_google_cloud_go_networkconnectivity",
        importpath = "cloud.google.com/go/networkconnectivity",
        sum = "h1:YsVhG71ZC4FkqCP2oCI55x/JeGFyd7738Lt8iNTrzJw=",
        version = "v1.16.1",
    )
    go_repository(
        name = "com_google_cloud_go_networkmanagement",
        importpath = "cloud.google.com/go/networkmanagement",
        sum = "h1:nnOEtE8HGt5zm0SZ0TUTAKAqSQaLdqALW56/0yBoyAg=",
        version = "v1.17.1",
    )
    go_repository(
        name = "com_google_cloud_go_networksecurity",
        importpath = "cloud.google.com/go/networksecurity",
        sum = "h1:JLJBFbxc8D7/OS81MyRoKhc2OvnVJxy5VMoQqqAhA7k=",
        version = "v0.10.3",
    )
    go_repository(
        name = "com_google_cloud_go_notebooks",
        importpath = "cloud.google.com/go/notebooks",
        sum = "h1:+9DrGJcZhCu6B2t0JJorekjIUBvg/KvBmXJYGmfvVvA=",
        version = "v1.12.3",
    )
    go_repository(
        name = "com_google_cloud_go_optimization",
        importpath = "cloud.google.com/go/optimization",
        sum = "h1:JwQjjoBZJpsoMQe/3mhVBMVZuSdagHg2pGOnwh2Jk+E=",
        version = "v1.7.3",
    )
    go_repository(
        name = "com_google_cloud_go_orchestration",
        importpath = "cloud.google.com/go/orchestration",
        sum = "h1:ug13kXJRdaZO94IgACVF4JwItuVnU0WFbHcJ5Z6ioNs=",
        version = "v1.11.3",
    )
    go_repository(
        name = "com_google_cloud_go_orgpolicy",
        importpath = "cloud.google.com/go/orgpolicy",
        sum = "h1:WFvgmjq/FO5GiXlhebltA9N14KdbLMcgG88ME+SWeBo=",
        version = "v1.14.2",
    )
    go_repository(
        name = "com_google_cloud_go_osconfig",
        importpath = "cloud.google.com/go/osconfig",
        sum = "h1:cyf1PMK5c2/WOIr5r2lxjH/XBJMA9P4zC8Tm10i0z3M=",
        version = "v1.14.3",
    )
    go_repository(
        name = "com_google_cloud_go_oslogin",
        importpath = "cloud.google.com/go/oslogin",
        sum = "h1:yomxnFPk+ye0zd0mJ15nn9fH4Ns7ex4xA3ll+u2q59A=",
        version = "v1.14.3",
    )
    go_repository(
        name = "com_google_cloud_go_phishingprotection",
        importpath = "cloud.google.com/go/phishingprotection",
        sum = "h1:T5mGFV0ggBKg3qt9myFRiGJu+nIUucuHLAtVpAuQ08I=",
        version = "v0.9.3",
    )
    go_repository(
        name = "com_google_cloud_go_policytroubleshooter",
        importpath = "cloud.google.com/go/policytroubleshooter",
        sum = "h1:ekIWI8JbKkpOfrgH/THGamQE/D16tcVBYJyrkseVcYI=",
        version = "v1.11.3",
    )
    go_repository(
        name = "com_google_cloud_go_privatecatalog",
        importpath = "cloud.google.com/go/privatecatalog",
        sum = "h1:fu2LABMi7CgZORQ2oNGbc0hoZ0FTqLkjGqIgAV/Kc7U=",
        version = "v0.10.4",
    )
    go_repository(
        name = "com_google_cloud_go_pubsub",
        importpath = "cloud.google.com/go/pubsub",
        sum = "h1:prYj8EEAAAwkp6WNoGTE4ahe0DgHoyJd5Pbop931zow=",
        version = "v1.45.3",
    )
    go_repository(
        name = "com_google_cloud_go_pubsublite",
        importpath = "cloud.google.com/go/pubsublite",
        sum = "h1:jLQozsEVr+c6tOU13vDugtnaBSUy/PD5zK6mhm+uF1Y=",
        version = "v1.8.2",
    )
    go_repository(
        name = "com_google_cloud_go_recaptchaenterprise_v2",
        importpath = "cloud.google.com/go/recaptchaenterprise/v2",
        sum = "h1:8tyxuDlUnyNsLd3Pf2qfdj6pjwE5UxQCLJdmPX8NIpk=",
        version = "v2.19.3",
    )
    go_repository(
        name = "com_google_cloud_go_recommendationengine",
        importpath = "cloud.google.com/go/recommendationengine",
        sum = "h1:kBpcYPx4ys4lrDGKp4OhP2uy8h7UjlmLW/qoO5Xb2bY=",
        version = "v0.9.3",
    )
    go_repository(
        name = "com_google_cloud_go_recommender",
        importpath = "cloud.google.com/go/recommender",
        sum = "h1:dVlOjxsbjuhlwu4MIcyPWe09qVcDqc419iOjdPl5RHk=",
        version = "v1.13.3",
    )
    go_repository(
        name = "com_google_cloud_go_redis",
        importpath = "cloud.google.com/go/redis",
        sum = "h1:ROQXi5dCDSJCVezt/2nD1g+Ym0T6sio3DIzZ56NgMZI=",
        version = "v1.17.3",
    )
    go_repository(
        name = "com_google_cloud_go_resourcemanager",
        importpath = "cloud.google.com/go/resourcemanager",
        sum = "h1:SHOMw0kX0xWratC5Vb5VULBeWiGlPYAs82kiZqNtWpM=",
        version = "v1.10.3",
    )
    go_repository(
        name = "com_google_cloud_go_resourcesettings",
        importpath = "cloud.google.com/go/resourcesettings",
        sum = "h1:13HOFU7v4cEvIHXSAQbinF4wp2Baybbq7q9FMctg1Ek=",
        version = "v1.8.3",
    )
    go_repository(
        name = "com_google_cloud_go_retail",
        importpath = "cloud.google.com/go/retail",
        sum = "h1:PT6CUlazIFIOLLJnV+bPBtiSH8iusKZ+FZRzZYFt2vk=",
        version = "v1.19.2",
    )
    go_repository(
        name = "com_google_cloud_go_run",
        importpath = "cloud.google.com/go/run",
        sum = "h1:aeVLygw0BGLH+Zbj8v3K3nEHvKlgoq+j8fcRJaYZtxY=",
        version = "v1.8.1",
    )
    go_repository(
        name = "com_google_cloud_go_scheduler",
        importpath = "cloud.google.com/go/scheduler",
        sum = "h1:p6+h8BoYJC+TvUijGBfORN6nuhOvJ3EwZ2H84CZ1ZEU=",
        version = "v1.11.3",
    )
    go_repository(
        name = "com_google_cloud_go_secretmanager",
        importpath = "cloud.google.com/go/secretmanager",
        sum = "h1:XVGHbcXEsbrgi4XHzgK5np81l1eO7O72WOXHhXUemrM=",
        version = "v1.14.3",
    )
    go_repository(
        name = "com_google_cloud_go_security",
        importpath = "cloud.google.com/go/security",
        sum = "h1:ya9gfY1ign6Yy25VMMMgZ9xy7D/TczDB0ElXcyWmEVE=",
        version = "v1.18.3",
    )
    go_repository(
        name = "com_google_cloud_go_securitycenter",
        importpath = "cloud.google.com/go/securitycenter",
        sum = "h1:H8UvBpcvs1OjI4jZuXX8xsN1IZo88a9PezHXkU2sGps=",
        version = "v1.35.3",
    )
    go_repository(
        name = "com_google_cloud_go_servicedirectory",
        importpath = "cloud.google.com/go/servicedirectory",
        sum = "h1:oFkCp6ti7fc7hzeROmOPQuPBHFqwyhcsv3Yrma28+uc=",
        version = "v1.12.3",
    )
    go_repository(
        name = "com_google_cloud_go_shell",
        importpath = "cloud.google.com/go/shell",
        sum = "h1:mjYgUsOtV3jl9xvDmcvlRRmA64deEPf52zOfuc68b/g=",
        version = "v1.8.3",
    )
    go_repository(
        name = "com_google_cloud_go_spanner",
        importpath = "cloud.google.com/go/spanner",
        sum = "h1:0bab8QDn6MNj9lNK6XyGAVFhMlhMU2waePPa6GZNoi8=",
        version = "v1.73.0",
    )
    go_repository(
        name = "com_google_cloud_go_speech",
        importpath = "cloud.google.com/go/speech",
        sum = "h1:qvURtJs7BQzQhbxWxwai0pT79S8KLVKJ/4W8igVkt1Y=",
        version = "v1.26.0",
    )
    go_repository(
        name = "com_google_cloud_go_storagetransfer",
        importpath = "cloud.google.com/go/storagetransfer",
        sum = "h1:W3v9A7MGBN7H9sAFstyciwP/1XEQhUhZfrjclmDnpMs=",
        version = "v1.12.1",
    )
    go_repository(
        name = "com_google_cloud_go_talent",
        importpath = "cloud.google.com/go/talent",
        sum = "h1:olv+s2g+LGXeJi+MYF1wI44/TwHaVnO0N7PiucVf5ZQ=",
        version = "v1.8.0",
    )
    go_repository(
        name = "com_google_cloud_go_texttospeech",
        importpath = "cloud.google.com/go/texttospeech",
        sum = "h1:YF/RdNb+jUEp22cIZCvqiFjfA5OxGE+Dxss3mhXU7oQ=",
        version = "v1.11.0",
    )
    go_repository(
        name = "com_google_cloud_go_tpu",
        importpath = "cloud.google.com/go/tpu",
        sum = "h1:PszqG+pvC7u/cv51GWQIN9M++jciIBr5vVn6/MWzU8I=",
        version = "v1.7.3",
    )
    go_repository(
        name = "com_google_cloud_go_trace",
        importpath = "cloud.google.com/go/trace",
        sum = "h1:c+I4YFjxRQjvAhRmSsmjpASUKq88chOX854ied0K/pE=",
        version = "v1.11.3",
    )
    go_repository(
        name = "com_google_cloud_go_translate",
        importpath = "cloud.google.com/go/translate",
        sum = "h1:XJ7LipYJi80BCgVk2lx1fwc7DIYM6oV2qx1G4IAGQ5w=",
        version = "v1.12.3",
    )
    go_repository(
        name = "com_google_cloud_go_video",
        importpath = "cloud.google.com/go/video",
        sum = "h1:C2FH+6yr6LCZC4fP0gm9FwJB/SRh5Ul88O5Sc/bL83I=",
        version = "v1.23.3",
    )
    go_repository(
        name = "com_google_cloud_go_videointelligence",
        importpath = "cloud.google.com/go/videointelligence",
        sum = "h1:zNTOUQyatGQtnCJ2dR3faRtpWQOlC8wszJqwG5CtwVM=",
        version = "v1.12.3",
    )
    go_repository(
        name = "com_google_cloud_go_vision_v2",
        importpath = "cloud.google.com/go/vision/v2",
        sum = "h1:dPvfDuPqPH+Yscf0f2f1RprvKkoo+N/j0a+IbLYX7Cs=",
        version = "v2.9.3",
    )
    go_repository(
        name = "com_google_cloud_go_vmmigration",
        importpath = "cloud.google.com/go/vmmigration",
        sum = "h1:dpCQq3pj2HnKdbvGTftdWymm3r4ovF7JW5z8xBcO2x4=",
        version = "v1.8.3",
    )
    go_repository(
        name = "com_google_cloud_go_vmwareengine",
        importpath = "cloud.google.com/go/vmwareengine",
        sum = "h1:TfuQr5j7qriINulUMotaC/+27SQaW2thIkF3Gb6VJ38=",
        version = "v1.3.3",
    )
    go_repository(
        name = "com_google_cloud_go_vpcaccess",
        importpath = "cloud.google.com/go/vpcaccess",
        sum = "h1:vxVaoFM64M/ht619c4wZNF0iq0QPaMWElOh7Ns4r41A=",
        version = "v1.8.3",
    )
    go_repository(
        name = "com_google_cloud_go_webrisk",
        importpath = "cloud.google.com/go/webrisk",
        sum = "h1:yh0v/5n49VO4/i9pYfDm1gLJUj1Ph3Xzegn8WvK9YRA=",
        version = "v1.10.3",
    )
    go_repository(
        name = "com_google_cloud_go_websecurityscanner",
        importpath = "cloud.google.com/go/websecurityscanner",
        sum = "h1:/uxhVCWKXzPw5pVfnBOVjaSiQ6Bm0tDExDOCLV40thw=",
        version = "v1.7.3",
    )
    go_repository(
        name = "com_google_cloud_go_workflows",
        importpath = "cloud.google.com/go/workflows",
        sum = "h1:lNFDMranJymDEB7cTI7DI9czbc1WU0RWY9KCEv9zuDY=",
        version = "v1.13.3",
    )
    go_repository(
        name = "dev_cel_expr",
        importpath = "cel.dev/expr",
        sum = "h1:NciYrtDRIR0lNCnH1LFJegdjspNx9fI59O7TWcua/W4=",
        version = "v0.19.1",
    )
    go_repository(
        name = "io_opentelemetry_go_auto_sdk",
        importpath = "go.opentelemetry.io/auto/sdk",
        sum = "h1:cH53jehLUN6UFLY71z+NDOiNJqDdPRaXzTel0sJySYA=",
        version = "v1.1.0",
    )
    go_repository(
        name = "io_opentelemetry_go_contrib_detectors_gcp",
        importpath = "go.opentelemetry.io/contrib/detectors/gcp",
        sum = "h1:JRxssobiPg23otYU5SbWtQC//snGVIM3Tx6QRzlQBao=",
        version = "v1.34.0",
    )
    go_repository(
        name = "io_opentelemetry_go_otel",
        importpath = "go.opentelemetry.io/otel",
        sum = "h1:zRLXxLCgL1WyKsPVrgbSdMN4c0FMkDAskSTQP+0hdUY=",
        version = "v1.34.0",
    )
    go_repository(
        name = "io_opentelemetry_go_otel_metric",
        importpath = "go.opentelemetry.io/otel/metric",
        sum = "h1:+eTR3U0MyfWjRDhmFMxe2SsW64QrZ84AOhvqS7Y+PoQ=",
        version = "v1.34.0",
    )
    go_repository(
        name = "io_opentelemetry_go_otel_sdk",
        importpath = "go.opentelemetry.io/otel/sdk",
        sum = "h1:95zS4k/2GOy069d321O8jWgYsW3MzVV+KuSPKp7Wr1A=",
        version = "v1.34.0",
    )
    go_repository(
        name = "io_opentelemetry_go_otel_sdk_metric",
        importpath = "go.opentelemetry.io/otel/sdk/metric",
        sum = "h1:5CeK9ujjbFVL5c1PhLuStg1wxA7vQv7ce1EK0Gyvahk=",
        version = "v1.34.0",
    )
    go_repository(
        name = "io_opentelemetry_go_otel_trace",
        importpath = "go.opentelemetry.io/otel/trace",
        sum = "h1:+ouXS2V8Rd4hp4580a8q23bg0azF2nI8cqLYnC8mh/k=",
        version = "v1.34.0",
    )
    go_repository(
        name = "net_starlark_go",
        importpath = "go.starlark.net",
        sum = "h1:xwwDQW5We85NaTk2APgoN9202w/l0DVGp+GZMfsrh7s=",
        version = "v0.0.0-20210223155950-e043a3d3c984",
    )
    go_repository(
        name = "org_golang_google_genproto",
        importpath = "google.golang.org/genproto",
        sum = "h1:387Y+JbxF52bmesc8kq1NyYIp33dnxCw6eiA7JMsTmw=",
        version = "v0.0.0-20250115164207-1a7da9e5054f",
    )
    go_repository(
        name = "org_golang_google_genproto_googleapis_api",
        importpath = "google.golang.org/genproto/googleapis/api",
        sum = "h1:GVIKPyP/kLIyVOgOnTwFOrvQaQUzOzGMCxgFUOEmm24=",
        version = "v0.0.0-20250106144421-5f5ef82da422",
    )
    go_repository(
        name = "org_golang_google_genproto_googleapis_bytestream",
        importpath = "google.golang.org/genproto/googleapis/bytestream",
        sum = "h1:7FlucM2tFADtEDnIlDrR12KdRqV48B1GSTU1U6uKSiY=",
        version = "v0.0.0-20251124214823-79d6a2a48846",
    )
    go_repository(
        name = "org_golang_google_genproto_googleapis_rpc",
        importpath = "google.golang.org/genproto/googleapis/rpc",
        sum = "h1:iK2jbkWL86DXjEx0qiHcRE9dE4/Ahua5k6V8OWFb//c=",
        version = "v0.0.0-20250313205543-e70fdf4c4cb4",
    )
    go_repository(
        name = "org_golang_google_grpc",
        importpath = "google.golang.org/grpc",
        sum = "h1:kF77BGdPTQ4/JZWMlb9VpJ5pa25aqvVqogsxNHHdeBg=",
        version = "v1.71.0",
    )
    go_repository(
        name = "org_golang_google_grpc_cmd_protoc_gen_go_grpc",
        importpath = "google.golang.org/grpc/cmd/protoc-gen-go-grpc",
        sum = "h1:F29+wU6Ee6qgu9TddPgooOdaqsxTMunOoj8KA5yuS5A=",
        version = "v1.5.1",
    )
    go_repository(
        name = "org_golang_google_protobuf",
        importpath = "google.golang.org/protobuf",
        sum = "h1:AYd7cD/uASjIL6Q9LiTjz8JLcrh/88q5UObnmY3aOOE=",
        version = "v1.36.10",
    )
    go_repository(
        name = "org_golang_x_crypto",
        importpath = "golang.org/x/crypto",
        sum = "h1:SHs+kF4LP+f+p14esP5jAoDpHU8Gu/v9lFRK6IT5imM=",
        version = "v0.39.0",
    )
    go_repository(
        name = "org_golang_x_mod",
        importpath = "golang.org/x/mod",
        sum = "h1:n7a+ZbQKQA/Ysbyb0/6IbB1H/X41mKgbhfv7AfG/44w=",
        version = "v0.25.0",
    )
    go_repository(
        name = "org_golang_x_net",
        importpath = "golang.org/x/net",
        sum = "h1:vBTly1HeNPEn3wtREYfy4GZ/NECgw2Cnl+nK6Nz3uvw=",
        version = "v0.41.0",
    )
    go_repository(
        name = "org_golang_x_oauth2",
        importpath = "golang.org/x/oauth2",
        sum = "h1:CY4y7XT9v0cRI9oupztF8AgiIu99L/ksR/Xp/6jrZ70=",
        version = "v0.25.0",
    )
    go_repository(
        name = "org_golang_x_sync",
        importpath = "golang.org/x/sync",
        sum = "h1:KWH3jNZsfyT6xfAfKiz6MRNmd46ByHDYaZ7KSkCtdW8=",
        version = "v0.15.0",
    )
    go_repository(
        name = "org_golang_x_sys",
        importpath = "golang.org/x/sys",
        sum = "h1:q3i8TbbEz+JRD9ywIRlyRAQbM0qF7hu24q3teo2hbuw=",
        version = "v0.33.0",
    )
    go_repository(
        name = "org_golang_x_telemetry",
        importpath = "golang.org/x/telemetry",
        sum = "h1:zf5N6UOrA487eEFacMePxjXAJctxKmyjKUsjA11Uzuk=",
        version = "v0.0.0-20240521205824-bda55230c457",
    )
    go_repository(
        name = "org_golang_x_term",
        importpath = "golang.org/x/term",
        sum = "h1:DR4lr0TjUs3epypdhTOkMmuF5CDFJ/8pOnbzMZPQ7bg=",
        version = "v0.32.0",
    )
    go_repository(
        name = "org_golang_x_text",
        importpath = "golang.org/x/text",
        sum = "h1:P42AVeLghgTYr4+xUnTRKDMqpar+PtX7KWuNQL21L8M=",
        version = "v0.26.0",
    )
    go_repository(
        name = "org_golang_x_tools",
        importpath = "golang.org/x/tools",
        sum = "h1:qIpSLOxeCYGg9TrcJokLBG4KFA6d795g0xkBkiESGlo=",
        version = "v0.34.0",
    )
    go_repository(
        name = "org_golang_x_tools_go_vcs",
        importpath = "golang.org/x/tools/go/vcs",
        sum = "h1:cOIJqWBl99H1dH5LWizPa+0ImeeJq3t3cJjaeOWUAL4=",
        version = "v0.1.0-deprecated",
    )
    go_repository(
        name = "org_golang_x_xerrors",
        importpath = "golang.org/x/xerrors",
        sum = "h1:go1bK/D/BFZV2I8cIQd1NKEZ+0owSTG1fDTci4IqFcE=",
        version = "v0.0.0-20200804184101-5ec99f83aff1",
    )
