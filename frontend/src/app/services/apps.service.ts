import { Injectable, inject } from '@angular/core';
import { HttpClient } from '@angular/common/http';

export interface Placeholder {
  name: string;
  kind: string;
  label?: string;
  required: boolean;
  default?: unknown;
  regex?: string;
  min_length?: number;
  max_length?: number;
}

export interface Port {
  container: number;
  host?: number;
}

export interface App {
  name: string;
  slug: string;
  description?: string;
  category?: string;
  image: string;
  status: string;
  ports: Port[];
  placeholders: Placeholder[];
}

export interface InstallResult {
  install_id: string;
  slug: string;
  status: string;
  url: string;
}

@Injectable({ providedIn: 'root' })
export class AppsService {
  private readonly http = inject(HttpClient);

  list() {
    return this.http.get<{ apps: App[]; base_domain: string }>('/api/v1/apps');
  }

  get(slug: string) {
    return this.http.get<{ app: App }>(`/api/v1/apps/${slug}`);
  }

  install(slug: string, values: Record<string, string>) {
    return this.http.post<InstallResult>('/api/v1/apps/install', { slug, values });
  }

  uninstall(slug: string) {
    return this.http.post<{ slug: string; status: string }>(`/api/v1/apps/${slug}/uninstall`, {});
  }
}
