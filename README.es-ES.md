# Trae Account Manager

<div align="center">

<img src="src-tauri/icons/icon.png" alt="Trae Account Manager" width="128" />

<br />

<p><b>Una potente herramienta de gestión de múltiples cuentas para Trae IDE con optimizaciones de rendimiento.</b></p>

![Version](https://img.shields.io/badge/version-0.1.0-green)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey)
![License](https://img.shields.io/badge/license-MIT-orange)

</div>

## 📖 Descripción General

**Trae Account Manager** es una aplicación de escritorio diseñada para que los usuarios de Trae IDE gestionen múltiples cuentas de manera eficiente. Cambie entre cuentas sin interrupciones, monitoree el uso en tiempo real y optimice su flujo de trabajo con funciones avanzadas de rendimiento.

Este proyecto está basado en [Yang-505/Trae-Account-Manager](https://github.com/Yang-505/Trae-Account-Manager) con mejoras significativas en rendimiento y arquitectura.

## ✨ Características Principales

- **Cambio de Cuenta en un Clic**: Gestiona automáticamente el proceso de Trae IDE y el estado de inicio de sesión.
- **Monitoreo de Uso en Tiempo Real**: Rastrea el consumo de tokens y las cuotas restantes.
- **Alto Rendimiento**: Paginación, desplazamiento virtual y almacenamiento en caché inteligente para listas grandes de cuentas.
- **Logging de Grado de Producción**: Registro consciente del entorno con Pino (formato legible en desarrollo, JSON en producción).
- **Comunicación WebSocket**: Actualizaciones de sesión en tiempo real con reconexión automática.
- **Gestión de Datos**: Importación/exportación de cuentas en formato JSON para facilitar el respaldo y el intercambio.

## 🚀 Primeros Pasos

### Requisitos Previos

- **Windows 10/11**, **macOS** o **Linux**
- **Trae IDE** instalado
- **Node.js 16+** (para desarrollo)

### Instalación

Descargue la última versión para su plataforma desde [Releases](https://github.com/911218sky/Trae-Account-Manager/releases).

### Compilar desde el Código Fuente

```bash
# Clonar repositorio
git clone https://github.com/911218sky/Trae-Account-Manager.git
cd Trae-Account-Manager

# Instalar dependencias
npm install

# Modo de desarrollo
npm run tauri dev

# Compilar versión de producción
npm run tauri build
```

## 💻 Uso

### 1. Configurar la ruta de Trae IDE

Abra **Settings** $\rightarrow$ Haga clic en **Auto Scan** o **Manual Setup** para localizar su archivo `Trae.exe`.

### 2. Agregar Cuenta

Haga clic en **Add Account** $\rightarrow$ Ingrese su token de Trae IDE $\rightarrow$ Haga clic en **Add**.

**Cómo obtener su token:**
1. Abra Trae IDE y presione `F12`
2. Vaya a `Application` $\rightarrow$ `Local Storage` $\rightarrow$ `vscode-webview://xxx`
3. Busque la clave que contiene `iCubeAuthInfo` y copie el valor del `token`

### 3. Cambiar Cuenta

Haga clic en **Switch** en cualquier tarjeta de cuenta $\rightarrow$ Confirme $\rightarrow$ La aplicación reiniciará automáticamente Trae IDE con la nueva cuenta.

### 4. Monitorear Uso

Vea las estadísticas de uso en tiempo real en el tablero o haga clic en **Details** para un historial de uso detallado.

## 🎯 Mejoras Implementadas

Este fork incluye mejoras significativas respecto al original:

### Optimizaciones de Rendimiento
- **Sistema de Paginación**: Carga de cuentas en lotes (50 por página).
- **Desplazamiento Virtual**: Renderiza solo los elementos visibles para un rendimiento fluido.
- **Caché Inteligente**: Caché de 5 minutos con invalidación automática.
- **Carga Perezosa (Lazy Loading)**: Carga automática al alcanzar el 80% del desplazamiento.

### Logging Avanzado
- **Integración con Pino**: Registro rápido y de bajo consumo de recursos.
- **Conciencia del Entorno**: Logs legibles en desarrollo, JSON estructurado en producción.
- **Salida a Archivo**: Soporte opcional para archivos de log.
- **Loggers por Módulo**: Registro organizado por componentes.

### Mejoras de Arquitectura
- **Modo Estricto de TypeScript**: Seguridad de tipos mejorada.
- **Capa de Servicio**: Servicios modulares (caché, logger, monitor de rendimiento).
- **Hooks Personalizados**: Hooks de React reutilizables para patrones comunes.
- **Límites de Error (Error Boundaries)**: Manejo de errores elegante.
- **Cliente WebSocket**: Comunicación en tiempo real con reconexión automática.

### Experiencia del Desarrollador
- **Suite de Pruebas**: Pruebas exhaustivas con Vitest.
- **CI/CD**: Lanzamientos automatizados mediante GitHub Actions.
- **Documentación**: Notas de implementación detalladas.
- **Guías de IA**: Estándares de desarrollo en AGENTS.md.

## 🛠️ Stack Tecnológico

- **Frontend**: React 19 + TypeScript + Vite
- **Backend**: Tauri 2 + Rust
- **Logging**: Pino
- **Tiempo Real**: WebSocket
- **Pruebas**: Vitest

## 📁 Estructura del Proyecto

```
Trae-Account-Manager/
├── src/                    # Código fuente del frontend
│   ├── components/        # Componentes de React
│   ├── services/          # Capa de servicio (API, caché, logger)
│   ├── hooks/             # Hooks personalizados de React
│   └── pages/             # Componentes de página
├── src-tauri/             # Backend de Tauri (Rust)
└── .github/workflows/     # Flujos de trabajo CI/CD
```

## 📂 Almacenamiento de Datos

Los datos de la aplicación se almacenan localmente:

- **Windows**: `%APPDATA%\com.sauce.trae-auto\`
- **macOS**: `~/Library/Application Support/com.sauce.trae-auto/`
- **Linux**: `~/.local/share/com.sauce.trae-auto/`

## 🔗 Integración

Exporte sus cuentas como JSON e impórtelas en otras herramientas o compártalas con los miembros de su equipo.

### Creación Automatizada de Cuentas

Para el registro automatizado de cuentas, visite [Trae-Account-Creator](https://github.com/911218sky/Trae-Account-Creator), una herramienta complementaria que automatiza el proceso de creación de cuentas de Trae IDE.

## ⚠️ Descargo de Responsabilidad

Este proyecto tiene fines exclusivamente educativos y de investigación.

- Úselo bajo su propio riesgo.
- Puede violar los términos de servicio del software.
- Los autores no se hacen responsables de ningún daño.
- No apto para uso comercial.

## 🤝 Contribuciones

¡Las contribuciones son bienvenidas! No dude en enviar problemas (issues) o solicitudes de extracción (pull requests).

```bash
# Fork y clonar
git checkout -b feature/AmazingFeature
git commit -m 'Add some AmazingFeature'
git push origin feature/AmazingFeature
# Abrir un Pull Request
```

## 📄 Licencia

Este proyecto está licenciado bajo la Licencia MIT.

## 💖 Créditos

- **Proyecto Original**: [Yang-505/Trae-Account-Manager](https://github.com/Yang-505/Trae-Account-Manager) por [@Yang-505](https://github.com/Yang-505)
- **Tauri**: [tauri.app](https://tauri.app/)
- **React**: [react.dev](https://react.dev/)
- **Pino**: [getpino.io](https://getpino.io/)

---

<div align="center">

Hecho con ❤️ por la comunidad

</div>
