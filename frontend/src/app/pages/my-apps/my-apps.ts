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

  uninstall(app: App) {
    this.appsService.uninstall(app.slug).subscribe({
      next: () => this.load(),
      error: (err) => {
        this.error = err?.error?.message ?? 'Uninstall failed';
      },
    });
  }
}
