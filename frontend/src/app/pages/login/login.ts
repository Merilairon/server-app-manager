import { Component, inject } from '@angular/core';
import { FormBuilder, ReactiveFormsModule, Validators } from '@angular/forms';
import { Router } from '@angular/router';
import { AuthService } from '../../services/auth.service';

@Component({
  selector: 'app-login',
  imports: [ReactiveFormsModule],
  templateUrl: './login.html',
  styleUrl: './login.scss',
})
export class Login {
  private readonly auth = inject(AuthService);
  private readonly router = inject(Router);
  private readonly fb = inject(FormBuilder);

  readonly form = this.fb.group({
    username: ['', Validators.required],
    password: ['', Validators.required],
  });

  error = '';
  loading = false;

  onSubmit() {
    if (this.form.invalid) return;
    const { username, password } = this.form.value;
    this.loading = true;
    this.auth.login(username ?? '', password ?? '').subscribe({
      next: () => {
        this.loading = false;
        this.router.navigate(['/']).then((ok) => {
          if (!ok) {
            this.error = 'Could not open dashboard. Please try again.';
          }
        });
      },
      error: (err) => {
        this.loading = false;
        this.error = err?.error?.message ?? 'Login failed';
      },
    });
  }
}
