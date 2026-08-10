import { Component, OnInit, inject, signal } from '@angular/core';
import { App, AppsService } from '../../services/apps.service';
import { AuthService } from '../../services/auth.service';
import { InstallModal } from '../../components/install-modal/install-modal';

@Component({
  selector: 'app-apps',
  imports: [InstallModal],
  templateUrl: './apps.html',
  styleUrl: './apps.scss',
})
export class Apps implements OnInit {
  private readonly appsService = inject(AppsService);
  protected readonly auth = inject(AuthService);

  apps = signal<App[]>([]);
  selected = signal<App | null>(null);
  loading = signal(true);
  error = '';

  ngOnInit() {
    this.appsService.list().subscribe({
      next: (res) => {
        this.apps.set(res.apps.filter((a) => a.status === 'available'));
        this.loading.set(false);
      },
      error: (err) => {
        this.loading.set(false);
        this.error = err?.error?.message ?? 'Failed to load app store';
      },
    });
  }

  select(app: App) {
    this.selected.set(app);
  }

  onDone() {
    this.selected.set(null);
    this.ngOnInit();
  }
}
