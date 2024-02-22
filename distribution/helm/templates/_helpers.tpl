{{/*
# 🐻‍❄️💐 ume: Easy, self-hostable, and flexible image host made in Rust
# Copyright 2021-2024 Noel Towa <cutie@floofy.dev>
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
*/}}

{{/* vim: set filetype=mustache: */}}

{{/*
Common labels
*/}}
{{- define "ume.labels" -}}
{{ include "ume.selectorLabels" . }}
k8s.noel.pink/managed-by: Helm
{{- end }}

{{- define "ume.selectorLabels" -}}
k8s.noel.pink/chart: {{ include "common.names.name" . }}
k8s.noel.pink/instance: {{ .Release.Name }}
{{- end }}

{{/*
Create the name of the service account to use
*/}}
{{- define "ume.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "ume.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}

{{/*
Default annotations
*/}}
{{- define "ume.annotations" -}}
k8s.noel.pink/component: imageserver
{{- if .Chart.AppVersion }}
k8s.noel.pink/version: {{ .Chart.AppVersion | quote }}
{{- end }}

{{- range $key, $val := .Values.global.annotations }}
    {{ $key }}: {{ $val | quote }}
{{- end }}
{{- end -}}
