# Security Policy

## Supported Versions

| Version | Supported          |
|----------|-------------------|
| 0.1.x   | ✓ Yes |
| < 0.1.0  | ✗ No               |

We support the current stable release and apply security fixes to it. We may backport important security fixes to the most recent minor version branch.

## Reporting a Vulnerability

If you discover a security vulnerability in UpdateHauler, please report it to us responsibly. We take security reports seriously and will address them promptly.

### How to Report

**Private Disclosure (Preferred)**

For security vulnerabilities, please do not open a public issue. Instead, send your report privately:

- **Email**: [security@updatehauler.dev](mailto:security@updatehauler.dev)
- **GitHub Security Advisory**: Use GitHub's private vulnerability reporting feature:
  1. Go to the [Security tab](https://github.com/franksplace/updatehauler/security)
  2. Click "Report a vulnerability"
  3. Follow the form to submit your report privately

### What to Include

Please include the following information in your report:

- **Description**: A detailed description of the vulnerability
- **Affected versions**: Which versions of UpdateHauler are affected
- **Steps to reproduce**: Clear, reproducible steps to demonstrate the vulnerability
- **Impact**: What can be achieved by exploiting the vulnerability (e.g., code execution, data exposure)
- **Proof of concept**: If possible, provide a minimal proof of concept
- **Suggested fix** (optional): If you have a suggested fix, please describe it

### What to Expect

After you submit a security report, you can expect:

1. **Initial Response**: We will acknowledge your report within 48 hours
2. **Investigation**: We will investigate the vulnerability and validate it
3. **Communication**: We will keep you informed of our progress
4. **Resolution**: We will work on a fix and release an update
5. **Disclosure**: We will coordinate public disclosure of the vulnerability

### Security Update Process

1. **Fix**: We develop a patch for the vulnerability
2. **Test**: We thoroughly test the fix
3. **Release**: We release a new version with the security fix
4. **Advisory**: We publish a security advisory on GitHub with details
5. **Credit**: We credit the reporter (with permission) in the advisory

### Timeline

- **Triage**: Within 48 hours of receiving a report
- **Initial Analysis**: Within 7 days
- **Fix Development**: Typically 2-4 weeks depending on severity
- **Public Disclosure**: After fix is released and users have time to update

## Security Best Practices

### For Users

1. **Keep Updated**: Always use the latest version of UpdateHauler
2. **Review Permissions**: Run UpdateHauler with minimal necessary permissions
3. **Audit Updates**: Review changelog before updating
4. **Use Dry-Run**: Test configuration with `--dry-run` flag before running updates
5. **Secure Config**: Protect your configuration files with appropriate file permissions

### For Developers

1. **Dependency Management**: Regularly update dependencies
2. **Code Review**: All changes go through peer review
3. **Static Analysis**: Use `cargo clippy` and security scanners
4. **Testing**: Comprehensive test coverage including edge cases
5. **Least Privilege**: Avoid running with unnecessary elevated privileges

## Known Security Considerations

### Package Updates

UpdateHauler executes external package manager commands:
- This means it trusts the package manager's security model
- Always verify the integrity of package manager installations
- Use official package repositories

### Scheduled Updates

When scheduling updates:
- Consider system security implications of automated updates
- Monitor logs for unexpected behavior
- Use `--dry-run` to preview changes before enabling schedule

### Configuration Files

- Sensitive information may be stored in configuration files
- Protect config files with appropriate permissions (`chmod 600`)
- Avoid committing config files with sensitive data to version control

### Dry-Run Mode

While dry-run mode is safer:
- It still executes detection commands
- Use with caution in production environments
- Review dry-run output before executing actual updates

## Security Features

UpdateHauler includes several security-related features:

- **Dry-Run Mode**: Preview changes without execution
- **Logging**: Comprehensive logging for audit trails
- **Configuration Validation**: YAML config validation before execution
- **Permission Awareness**: Respects system permissions and user capabilities
- **Error Handling**: Graceful error handling without exposing system details

## Vulnerability Response Levels

We classify vulnerabilities based on severity:

| Severity | Response Time  | Public Disclosure |
|-----------|---------------|-------------------|
| Critical  | < 48 hours    | Immediate         |
| High      | < 72 hours    | < 7 days          |
| Medium    | < 1 week       | < 2 weeks         |
| Low        | < 2 weeks      | Next release      |

### Severity Definitions

- **Critical**: Exploitable by unauthenticated users, results in system compromise, data exposure, or denial of service
- **High**: Exploitable with authentication, significant impact on confidentiality, integrity, or availability
- **Medium**: Some exploitation difficulty, limited impact, or requires specific conditions
- **Low**: Minor impact, difficult to exploit, or requires significant privileges

## Receiving Security Updates

### GitHub Security Advisories

GitHub users who watch this repository will automatically receive security advisories. Ensure notifications are enabled for security alerts.

### Release Monitoring

Monitor releases for security-related updates by:
- Watching the repository
- Subscribing to the [release RSS feed](https://github.com/franksplace/updatehauler/releases.atom)
- Checking the [CHANGELOG.md](CHANGELOG.md) for security fix notes

## Contact

For security questions or concerns not related to a vulnerability report:

- **Email**: [security@updatehauler.dev](mailto:security@updatehauler.dev)
- **GitHub**: [Security Discussions](https://github.com/franksplace/updatehauler/security)

## Acknowledgments

We thank security researchers and users who report vulnerabilities responsibly. Your help makes UpdateHauler more secure for everyone.

## Resources

- [GitHub Security Best Practices](https://docs.github.com/en/code-security/securing-your-software-supply-chain/about-dependency-management-and-supply-chain-security)
- [Rust Security Best Practices](https://doc.rust-lang.org/reference/introduction.html#memory-safety)
- [OWASP Testing Guide](https://owasp.org/www-project-web-security-testing-guide/)
