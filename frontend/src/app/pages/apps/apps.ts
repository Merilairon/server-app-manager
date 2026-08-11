import { Component, OnInit, computed, inject, signal } from '@angular/core';
import { App, AppsService } from '../../services/apps.service';
import { AuthService } from '../../services/auth.service';
import { InstallModal } from '../../components/install-modal/install-modal';

@Component({
  selector: 'app-apps',
  standalone: true,
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
  filter = signal<string>('All');
  sortBy = signal<'name' | 'category'>('name');

  categories = computed(() => {
    const cats = this.apps().map((a) => a.category ?? 'Other');
    const unique = Array.from(new Set(cats)).sort();
    return ['All', ...unique];
  });

  filteredApps = computed(() => {
    let list = this.apps();
    const active = this.filter();
    if (active !== 'All') {
      list = list.filter((a) => (a.category ?? 'Other') === active);
    }
    const key = this.sortBy();
    return [...list].sort((a, b) => {
      if (key === 'category') {
        const byCat = (a.category ?? 'Other')
          .toLowerCase()
          .localeCompare((b.category ?? 'Other').toLowerCase());
        if (byCat !== 0) return byCat;
      }
      return a.name.toLowerCase().localeCompare(b.name.toLowerCase());
    });
  });

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

  setFilter(category: string) {
    this.filter.set(category);
  }

  setSort(value: string) {
    this.sortBy.set(value as 'name' | 'category');
  }
}
