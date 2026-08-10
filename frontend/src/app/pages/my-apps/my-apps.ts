import { Component, OnInit, inject, signal } from '@angular/core';
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

  ngOnInit() {
    this.load();
  }

  load() {
    this.loading.set(true);
    this.appsService.list().subscribe({
      next: (res) => {
        this.apps.set(res.apps.filter((a) => a.status === 'installed'));
        this.loading.set(false);
      },
      error: (err) => {
        this.loading.set(false);
        this.error = err?.error?.message ?? 'Failed to load apps';
      },
    });
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
