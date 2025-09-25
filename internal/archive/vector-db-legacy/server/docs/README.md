# OmenDB Server Documentation

**Enterprise vector database server documentation**

## 🚨 **AI Agent Quick Start**

**Read these files for server development context:**
1. **`/CLAUDE.md`** - Server development rules, architecture, enterprise features (2 min)
2. **`/README.md`** - Server package overview, installation, quick start (1 min)

**Then start server development with full context.**

## 📁 **Documentation Structure**

### **Root Level (Server Package Info)**
```
/CLAUDE.md      - Server development guide, architecture, enterprise features
/README.md      - Server package overview, installation, usage examples
/pyproject.toml - Server package configuration and dependencies
```

### **docs/ (Server Reference Documentation)**
```
docs/
├── api/            - Server API documentation
│   ├── rest-endpoints.md  - HTTP API specification
│   ├── grpc-service.md    - gRPC protocol definitions
│   └── authentication.md - JWT and API key management
├── monitoring/     - Observability and metrics
│   ├── setup.md           - Prometheus/OpenTelemetry setup
│   ├── metrics.md         - Available metrics reference
│   └── troubleshooting.md - Monitoring issues
├── mlops/          - Vector lifecycle management
│   ├── versioning.md      - Vector version control
│   ├── ab-testing.md      - Model comparison workflows
│   └── drift-detection.md - Vector quality monitoring
├── deployment/     - Production deployment
│   ├── production.md      - Docker/Kubernetes deployment
│   ├── configuration.md   - Environment setup
│   └── backup-restore.md  - Data protection strategies
├── config/         - Configuration reference
│   ├── server-yaml.md     - Complete server.yaml reference
│   ├── environment.md     - Runtime configuration
│   └── security.md        - Authentication settings
└── dev/            - Server development
    ├── setup.md           - Development environment
    ├── architecture.md    - Server architecture overview
    └── troubleshooting/   - Development issues
```

## 🎯 **Design Principles**

### **Server-First Architecture**
- **Enterprise focus**: Documentation emphasizes server/enterprise features
- **Embedded dependency**: Clear separation between embedded core and server features
- **Clean architecture**: Server depends on embedded submodule, not embedded code

### **Enterprise Documentation**
- **API specifications**: Complete REST/gRPC endpoint documentation
- **Production deployment**: Docker, Kubernetes, monitoring setup guides
- **Enterprise features**: MLOps, monitoring, authentication, scaling

### **Clear Dependency Model**
- **Embedded core**: Via git submodule at `embedded/`
- **Server features**: All enterprise functionality in `server/`
- **Import pattern**: `from embedded.omendb import DB` + server modules

## 📋 **Quick Navigation**

### **For AI Agents**
Start with `/CLAUDE.md` for server development context, then reference specific docs/ sections as needed.

### **For Server Operators**
1. [Installation Guide](../README.md#installation) - Install server package
2. [Configuration](config/server-yaml.md) - Set up server.yaml
3. [Deployment](deployment/production.md) - Production deployment

### **For API Users**
1. [REST API](api/rest-endpoints.md) - HTTP endpoints
2. [gRPC API](api/grpc-service.md) - Protocol buffer services
3. [Authentication](api/authentication.md) - API security

### **For Enterprise Features**
1. [Monitoring Setup](monitoring/setup.md) - Observability stack
2. [MLOps Integration](mlops/versioning.md) - Vector lifecycle management
3. [Production Deployment](deployment/production.md) - Enterprise deployment

### **For Developers**
1. [Development Setup](dev/setup.md) - Server development environment
2. [Architecture Overview](dev/architecture.md) - Server design patterns
3. [Troubleshooting](dev/troubleshooting/) - Common development issues

## ⚠️ **Server Context**

**This is the enterprise server repository:**
- **Server features**: REST/gRPC APIs, monitoring, MLOps, enterprise tools
- **Embedded dependency**: Uses public omendb via git submodule
- **Enterprise focus**: Documentation emphasizes production deployment and enterprise features

**If you find embedded-specific information:**
1. **CLAUDE.md wins** - Authoritative source for server development
2. **Check server/ directory** - All server features are in server/
3. **Embedded reference** - Check embedded/ submodule for core functionality

---

**This documentation focuses on enterprise server features built on the high-performance omendb embedded core.**