import { Component, OnInit, computed, inject, signal } from '@angular/core';
import { App, AppsService } from '../../services/apps.service';

@Component({
  selector: 'app-my-apps',
  imports: [],
  templateUrl: './my-apps.html',
  styleUrl: './my-apps.scss',
})
export class MyApps implements OnInit {
  private readonly appsService = inject(AppsService);

  apps = signal<App[]>([]);
  loading = signal(true);
  error = '';
  baseDomain = '';
  statusFilter = signal<string>('All');

  statuses = computed(() => {
    const all = this.apps().map((a) => a.status);
    const unique = Array.from(new Set(all)).sort();
    return ['All', ...unique];
  });

  filteredApps = computed(() => {
    let list = this.apps();
    const active = this.statusFilter();
    if (active !== 'All') {
      list = list.filter((a) => a.status === active);
    }
    return [...list].sort((a, b) => a.name.toLowerCase().localeCompare(b.name.toLowerCase()));
  });

  ngOnInit() {
    this.load();
  }

  load() {
    this.loading.set(true);
    this.appsService.list().subscribe({
      next: (res) => {
        this.apps.set(res.apps.filter((a) => a.status !== 'available'));
        this.baseDomain = res.base_domain;
        this.loading.set(false);
      },
      error: (err) => {
        this.loading.set(false);
        this.error = err?.error?.message ?? 'Failed to load apps';
      },
    });
  }

  setStatus(status: string) {
    this.statusFilter.set(status);
  }

  appUrl(app: App): string {
    const hostPort = app.ports.find((p) => p.host != null)?.host;
    const localDomain = this.baseDomain === 'localhost' || this.baseDomain.endsWith('.local');
    if (hostPort != null && localDomain) {
      return `http://localhost:${hostPort}`;
    }
    return `https://${app.slug}.${this.baseDomain}`;
  }

  uninstall(app: App) {
    this.appsService.uninstall(app.slug).subscribe({
      next: () => this.load(),
      error: (err) => {
        this.error = err?.error?.message ?? 'Uninstall failed';
      },
    });
  }

  statusBadgeClass(status: string): string {
    const map: Record<string, string> = {
      running: 'badge badge--ok',
      healthy: 'badge badge--ok',
      starting: 'badge badge--warn',
      stopped: 'badge badge--neutral',
      unhealthy: 'badge badge--bad',
    };
    return map[status.toLowerCase()] ?? 'badge badge--neutral';
  }
}
