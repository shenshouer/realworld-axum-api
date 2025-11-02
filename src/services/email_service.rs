use std::env;

use lettre::{
    Message, SmtpTransport, Transport,
    message::{Mailbox, header::ContentType},
    transport::smtp::authentication::Credentials,
};
use tracing::info;

pub struct EmailService {
    mailer: SmtpTransport,
    from_email: Mailbox,
}

impl EmailService {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        println!("Initializing email service...");
        let smtp_host = env::var("SMTP_HOST").expect("SMTP_HOST must be set");
        let smtp_port: u16 = env::var("SMTP_PORT")
            .expect("SMTP_PORT must be set")
            .parse()
            .expect("SMTP_PORT must be a valid number");
        let smtp_username = env::var("SMTP_USERNAME").expect("SMTP_USERNAME must be set");
        let smtp_password = env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD must be set");
        let from_email_str = env::var("SMTP_FROM_EMAIL").expect("SMTP_FROM_EMAIL must be set");
        let from_name = env::var("SMTP_FROM_NAME").expect("SMTP_FROM_NAME must be set");
        let credentials = Credentials::new(smtp_username, smtp_password);
        let mailer = SmtpTransport::starttls_relay(&smtp_host)?
            .port(smtp_port)
            .credentials(credentials)
            .build();
        let from_email = format!("{} <{}>", from_name, from_email_str)
            .parse()
            .expect("Invalid from email format");
        Ok(Self { mailer, from_email })
    }

    pub async fn send_verification_email(
        &self,
        to_email: &str,
        username: &str,
        verification_token: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let base_url = env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
        let verification_link = format!(
            "{}/api/auth/verify-email?token={}",
            base_url, verification_token
        );
        let html_body = format!(
            r#"
            <!DOCTYPE html>
            <html>
            <head>
                <style>
                    body {{ font-family: Arial, sans-serif; line-height: 1.6; color: #333; }}
                    .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
                    // ... more styles ...
                </style>
            </head>
            <body>
                <div class="container">
                    <div class="header">
                        <h1>Welcome to YourApp!</h1>
                    </div>
                    <div class="content">
                        <h2>Hi {}!</h2>
                        <p>Thanks for signing up! We're excited to have you on board.</p>
                        <p>Please verify your email address by clicking the button below:</p>
                        <div style="text-align: center;">
                            <a href="{}" class="button">Verify Email Address</a>
                        </div>
                        <p>Or copy and paste this link into your browser:</p>
                        <p style="background-color: #eee; padding: 10px; word-break: break-all;">{}</p>
                        <p><strong>This link will expire in 24 hours.</strong></p>
                        <p>If you didn't create an account, please ignore this email.</p>
                    </div>
                    <div class="footer">
                        <p>© 2024 YourApp. All rights reserved.</p>
                    </div>
                </div>
            </body>
            </html>
            "#,
            username, verification_link, verification_link
        );
        let email = Message::builder()
            .from(self.from_email.clone())
            .to(to_email.parse()?)
            .subject("Verify Your Email Address")
            .header(ContentType::TEXT_HTML)
            .body(html_body)?;
        self.mailer.send(&email)?;
        info!("Verification email sent to {}", to_email);
        info!("Verification link: {}", verification_link);

        Ok(())
    }

    pub async fn send_password_reset_email(
        &self,
        to_email: &str,
        username: &str,
        reset_token: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let base_url = env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
        let reset_link = format!("{}/api/auth/reset-password?token={}", base_url, reset_token);

        let html_body = format!(
            r#"
            <!DOCTYPE html>
            <html>
            <head>
                <style>
                    body {{ font-family: Arial, sans-serif; line-height: 1.6; color: #333; }}
                    .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
                    .header {{ background-color: #f8d7da; color: #721c24; padding: 20px; text-align: center; border-radius: 5px 5px 0 0; }}
                    .content {{ background-color: #fff; padding: 30px; border: 1px solid #ddd; }}
                    .button {{ display: inline-block; padding: 12px 24px; background-color: #dc3545; color: white; text-decoration: none; border-radius: 5px; margin: 20px 0; }}
                    .footer {{ text-align: center; margin-top: 20px; color: #666; font-size: 12px; }}
                    .warning {{ background-color: #fff3cd; border-left: 4px solid #ffc107; padding: 12px; margin: 20px 0; }}
                </style>
            </head>
            <body>
                <div class="container">
                    <div class="header">
                        <h1>Password Reset Request</h1>
                    </div>
                    <div class="content">
                        <h2>Hi {}!</h2>
                        <p>We received a request to reset your password. If you didn't make this request, you can safely ignore this email.</p>
                        <p>To reset your password, click the button below:</p>
                        <div style="text-align: center;">
                            <a href="{}" class="button">Reset Password</a>
                        </div>
                        <p>Or copy and paste this link into your browser:</p>
                        <p style="background-color: #eee; padding: 10px; word-break: break-all;">{}</p>
                        <div class="warning">
                            <p><strong>⚠️ Security Notice:</strong></p>
                            <ul>
                                <li>This link will expire in 1 hour</li>
                                <li>The link can only be used once</li>
                                <li>If you didn't request this reset, someone may be trying to access your account</li>
                            </ul>
                        </div>
                        <p>After clicking the link, you'll be able to create a new password for your account.</p>
                    </div>
                    <div class="footer">
                        <p>© 2024 AxumAPI. All rights reserved.</p>
                        <p>If you have security concerns, please contact our support team immediately.</p>
                    </div>
                </div>
            </body>
            </html>
            "#,
            username, reset_link, reset_link
        );

        let email = Message::builder()
            .from(self.from_email.clone())
            .to(to_email.parse()?)
            .subject("Reset Your Password")
            .header(ContentType::TEXT_HTML)
            .body(html_body)?;

        self.mailer.send(&email)?;

        info!("Password reset email sent to {}", to_email);
        info!("Reset link: {}", reset_link);

        Ok(())
    }

    pub async fn send_security_alert(
        &self,
        to_email: &str,
        username: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let html_body = format!(
            r#"
            <!DOCTYPE html>
            <html>
            <head>
                <style>
                    body {{ font-family: Arial, sans-serif; line-height: 1.6; color: #333; }}
                    .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
                    .header {{ background-color: #dc3545; color: white; padding: 20px; text-align: center; }}
                    .content {{ background-color: #f9f9f9; padding: 30px; border-radius: 5px; margin-top: 20px; }}
                    .alert-box {{ background-color: #fff3cd; border-left: 4px solid #ffc107; padding: 15px; margin: 20px 0; }}
                    .action-box {{ background-color: #d1ecf1; border-left: 4px solid #0c5460; padding: 15px; margin: 20px 0; }}
                    .footer {{ text-align: center; margin-top: 20px; color: #666; font-size: 12px; }}
                </style>
            </head>
            <body>
                <div class="container">
                    <div class="header">
                        <h1>Security Alert</h1>
                    </div>
                    <div class="content">
                        <h2>Hi {}!</h2>
                        <p>We detected suspicious activity on your account.</p>

                        <div class="alert-box">
                            <h3>What Happened?</h3>
                            <p>Someone attempted to use an old access token that had already been exchanged for a new one.</p>
                            <p>This usually means your token was stolen and someone else is trying to access your account.</p>
                        </div>

                        <div class="action-box">
                            <h3>What We Did</h3>
                            <ul>
                                <li>Blocked the suspicious request</li>
                                <li>Logged you out of all devices</li>
                                <li>Your account is now secure</li>
                            </ul>
                        </div>

                        <h3>What You Should Do</h3>
                        <ol>
                            <li><strong>Login again</strong> with your password</li>
                            <li><strong>Review recent activity</strong> on your account</li>
                            <li><strong>Change your password</strong> if you suspect compromise</li>
                            <li><strong>Enable 2FA</strong> if available (coming soon!)</li>
                        </ol>

                        <p><strong>When did this happen?</strong><br>
                        Just now - we detected and blocked it immediately.</p>

                        <p><strong>What if this wasn't you?</strong><br>
                        This is expected behavior if you were logged in on multiple devices. However, if you weren't actively using the app, someone may have your token.</p>

                        <p>If you have any questions or concerns, please contact our support team.</p>
                    </div>
                    <div class="footer">
                        <p>© 2024 AxumAPI. All rights reserved.</p>
                        <p>This is an automated security alert. Please do not reply to this email.</p>
                    </div>
                </div>
            </body>
            </html>
            "#,
            username
        );

        let email = Message::builder()
            .from(self.from_email.clone())
            .to(to_email.parse()?)
            .subject("Security Alert: Suspicious Activity Detected")
            .header(ContentType::TEXT_HTML)
            .body(html_body)?;

        self.mailer.send(&email)?;

        println!("Security alert sent to {}", to_email);

        Ok(())
    }
}
