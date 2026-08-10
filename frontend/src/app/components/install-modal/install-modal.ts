import { Component, EventEmitter, Input, OnInit, Output, inject } from '@angular/core';
import { FormBuilder, FormGroup, ReactiveFormsModule, Validators } from '@angular/forms';
import { App, AppsService } from '../../services/apps.service';

@Component({
  selector: 'app-install-modal',
  imports: [ReactiveFormsModule],
  templateUrl: './install-modal.html',
  styleUrl: './install-modal.scss',
})
export class InstallModal implements OnInit {
  private readonly fb = inject(FormBuilder);
  private readonly apps = inject(AppsService);

  @Input({ required: true }) app!: App;
  @Output() close = new EventEmitter<void>();
  @Output() done = new EventEmitter<void>();

  form = new FormGroup({});
  loading = false;
  error = '';
  step: 'configure' | 'install' | 'done' = 'configure';
  result: { install_id: string; status: string; url: string } | null = null;

  ngOnInit() {
    const group: Record<string, unknown> = {};
    for (const p of this.app.placeholders) {
      const value = p.default != null ? String(p.default) : '';
      const validators = p.required ? [Validators.required] : [];
      if (p.regex) {
        validators.push(Validators.pattern(p.regex));
      }
      group[p.name] = [value, validators];
    }
    this.form = this.fb.group(group);
  }

  fieldType(kind: string): string {
    if (kind === 'secret') return 'password';
    if (kind === 'integer') return 'number';
    return 'text';
  }

  install() {
    if (this.form.invalid) return;
    this.loading = true;
    this.error = '';
    this.step = 'install';

    const values: Record<string, string> = {};
    for (const [key, value] of Object.entries(this.form.value)) {
      values[key] = value != null ? String(value) : '';
    }

    this.apps.install(this.app.slug, values).subscribe({
      next: (res) => {
        this.result = res;
        this.loading = false;
        this.step = 'done';
      },
      error: (err) => {
        this.loading = false;
        this.step = 'configure';
        this.error = err?.error?.message ?? 'Installation failed';
      },
    });
  }
}
