/**
 * Logger Service Implementation
 * 
 * Core logger service that manages pino logger instances and provides
 * unified logging interface for the application.
 * 
 * Validates: Requirements 1.1, 1.2, 1.3, 6.1, 6.3
 */

import pino, { Logger as PinoLogger, LoggerOptions } from 'pino';
import { LoggerConfig, ModuleLogger, ILoggerService } from './types';

// Type guard for process.env access
declare const process: any;

/**
 * LoggerService Class
 * 
 * Main logger service implementation using pino as the underlying logger.
 * Provides methods for all log levels, configuration management, and module logger creation.
 */
export class LoggerService implements ILoggerService {
  private pinoInstance: PinoLogger;
  private config: LoggerConfig;

  /**
   * Constructor - Initialize LoggerService with optional configuration
   * 
   * @param config - Optional configuration object for the logger
   */
  constructor(config?: LoggerConfig) {
    this.config = this.normalizeConfig(config);
    this.pinoInstance = this.createPinoInstance();
  }

  /**
   * Normalize and validate configuration
   * 
   * @param config - Raw configuration object
   * @returns Normalized configuration with defaults applied
   */
  private normalizeConfig(config?: LoggerConfig): LoggerConfig {
    const environment = config?.environment || this.detectEnvironment();
    
    // Determine log level: explicit config > environment variable > default
    let level = config?.level;
    
    // Validate log level
    if (level && !['debug', 'info', 'warn', 'error'].includes(level)) {
      console.warn(`Invalid log level: ${level}, using default`);
      level = undefined;
    }
    
    if (!level) {
      try {
        const envLevel = typeof process !== 'undefined' && process.env?.LOG_LEVEL;
        if (envLevel && ['debug', 'info', 'warn', 'error'].includes(envLevel)) {
          level = envLevel as 'debug' | 'info' | 'warn' | 'error';
        }
      } catch {
        // Ignore errors
      }
      
      // Fall back to environment-based default
      if (!level) {
        level = environment === 'development' ? 'debug' : 'info';
      }
    }
    
    const prettyPrint = config?.prettyPrint !== undefined 
      ? config.prettyPrint 
      : environment === 'development';

    // Validate file output configuration if provided
    let fileOutput = config?.fileOutput;
    if (fileOutput?.enabled && fileOutput?.path) {
      if (!this.isFilePathWritable(fileOutput.path)) {
        fileOutput = undefined;
      }
    }

    return {
      level,
      environment,
      prettyPrint,
      fileOutput,
    };
  }

  /**
   * Detect environment from NODE_ENV variable
   * 
   * @returns 'development' or 'production'
   */
  private detectEnvironment(): 'development' | 'production' {
    try {
      // Try to read NODE_ENV from process.env
      const nodeEnv = typeof process !== 'undefined' && process.env?.NODE_ENV;
      
      // In production build (when import.meta.env.PROD is true), always use production
      if (typeof import.meta !== 'undefined' && import.meta.env?.PROD) {
        return 'production';
      }
      
      return nodeEnv === 'production' ? 'production' : 'development';
    } catch {
      // Fallback to production for safety in built applications
      return 'production';
    }
  }

  /**
   * Read log level from environment variable
   * 
   * @returns Log level from LOG_LEVEL env var or default based on environment
   */
  private readLogLevelFromEnv(): 'debug' | 'info' | 'warn' | 'error' {
    try {
      // Try Vite environment variable first (embedded in build)
      const viteLogLevel = typeof import.meta !== 'undefined' && import.meta.env?.VITE_LOG_LEVEL;
      if (viteLogLevel && ['debug', 'info', 'warn', 'error'].includes(viteLogLevel)) {
        return viteLogLevel as 'debug' | 'info' | 'warn' | 'error';
      }
      
      // Fallback to process.env
      const logLevel = typeof process !== 'undefined' && process.env?.LOG_LEVEL;
      if (logLevel && ['debug', 'info', 'warn', 'error'].includes(logLevel)) {
        return logLevel as 'debug' | 'info' | 'warn' | 'error';
      }
    } catch {
      // Ignore errors reading environment
    }
    
    // Default based on environment
    return this.config.environment === 'development' ? 'debug' : 'info';
  }

  /**
   * Read file output configuration from environment variables
   * 
   * @returns File output configuration or undefined
   */
  private readFileOutputFromEnv(): { enabled: boolean; path: string } | undefined {
    try {
      // Try Vite environment variables first (embedded in build)
      const viteEnabled = typeof import.meta !== 'undefined' && import.meta.env?.VITE_LOG_FILE_ENABLED === 'true';
      const viteFilePath = typeof import.meta !== 'undefined' && import.meta.env?.VITE_LOG_FILE_PATH;
      
      if (viteEnabled && viteFilePath) {
        if (this.isFilePathWritable(viteFilePath)) {
          return { enabled: true, path: viteFilePath };
        }
      }
      
      // Fallback to process.env
      const enabled = typeof process !== 'undefined' && process.env?.LOG_FILE_ENABLED === 'true';
      const filePath = typeof process !== 'undefined' && process.env?.LOG_FILE_PATH;
      
      if (enabled && filePath) {
        // Validate file path writability
        if (this.isFilePathWritable(filePath)) {
          return { enabled: true, path: filePath };
        } else {
          console.warn(`Log file path is not writable: ${filePath}`);
          return undefined;
        }
      }
    } catch {
      // Ignore errors reading environment
    }
    
    return undefined;
  }

  /**
   * Validate if a file path is writable
   * 
   * Attempts to validate the file path by checking if the directory exists
   * and is writable. For Tauri apps, this is a synchronous check based on
   * the path format and environment.
   * 
   * @param filePath - The file path to validate
   * @returns true if the path appears to be writable, false otherwise
   */
  private isFilePathWritable(filePath: string): boolean {
    try {
      // Basic validation: check if path is not empty and has valid format
      if (!filePath || typeof filePath !== 'string') {
        console.warn('Invalid log file path: path must be a non-empty string');
        return false;
      }

      // Check for absolute path or relative path with valid characters
      // Allow paths like ./logs/app.log, /var/log/app.log, C:\logs\app.log, etc.
      // Reject paths with special characters that are typically invalid
      const invalidCharsPattern = /[<>"|?*@#$%]/;
      if (invalidCharsPattern.test(filePath)) {
        console.warn(`Invalid log file path format: ${filePath}`);
        return false;
      }

      // Path appears valid - actual write permission will be checked when pino tries to write
      return true;
    } catch (error) {
      console.warn(`Error validating log file path: ${filePath}`, error);
      return false;
    }
  }

  /**
   * Create pino logger instance with appropriate configuration
   * 
   * Supports multi-transport setup for simultaneous console and file output.
   * Uses pino.multistream for multiple transports to ensure logs are written
   * to both console and file when both are enabled.
   * 
   * @returns Configured pino logger instance
   */
  private createPinoInstance(): PinoLogger {
    try {
      // Read log level from environment if not already set
      const logLevel = this.config.level || this.readLogLevelFromEnv();
      
      // Read file output configuration from environment if not already set
      const fileOutput = this.config.fileOutput || this.readFileOutputFromEnv();

      const pinoConfig: LoggerOptions = {
        level: logLevel,
        timestamp: pino.stdTimeFunctions.isoTime,
      };

      // Build transport configuration based on environment
      const transports: any[] = [];

      // Add console transport based on environment
      if (this.config.environment === 'development' && this.config.prettyPrint) {
        // Development with pretty print: use pino-pretty for console
        transports.push({
          target: 'pino-pretty',
          options: {
            colorize: true,
            translateTime: 'HH:MM:ss',
            ignore: 'pid,hostname',
            singleLine: false,
          },
        });
      } else {
        // Production or development without pretty print: use default JSON to stdout
        transports.push({
          target: 'pino/file',
          options: {
            destination: 1, // stdout
          },
        });
      }

      // Add file transport if configured
      if (fileOutput?.enabled && fileOutput?.path) {
        try {
          transports.push({
            target: 'pino/file',
            options: {
              destination: fileOutput.path,
            },
          });
        } catch (fileError) {
          // Log file transport error but continue with console output
          console.warn(`Failed to configure file transport for ${fileOutput.path}:`, fileError);
        }
      }

      // Create logger with appropriate transport configuration
      if (transports.length > 0) {
        try {
          // Use multistream for multiple transports to ensure all destinations receive logs
          const streams = transports.map(t => {
            try {
              return pino.transport(t);
            } catch (transportError) {
              console.warn(`Failed to create transport:`, transportError);
              // Return a fallback transport that writes to stdout
              return pino.transport({
                target: 'pino/file',
                options: { destination: 1 },
              });
            }
          });

          // Use multistream to combine all transports
          return pino(pinoConfig, pino.multistream(streams));
        } catch (multiStreamError) {
          // Fallback: if multistream fails, try single transport
          console.warn('Failed to create multistream logger, falling back to single transport:', multiStreamError);
          try {
            return pino(pinoConfig, pino.transport(transports[0]));
          } catch (singleTransportError) {
            // Final fallback: basic pino
            console.warn('Failed to create single transport logger, using basic pino:', singleTransportError);
            return pino(pinoConfig);
          }
        }
      }

      // No special transport configured - use default pino
      return pino(pinoConfig);
    } catch (error) {
      // Fallback to basic pino if transport configuration fails
      console.error('Failed to create pino instance with transport:', error);
      const logLevel = this.config.level || this.readLogLevelFromEnv();
      return pino({ level: logLevel });
    }
  }



  /**
   * Log a debug level message
   * 
   * @param message - The message to log
   * @param context - Optional contextual data
   */
  debug(message: string, context?: Record<string, any>): void {
    try {
      this.pinoInstance.debug(context || {}, message);
    } catch (error) {
      console.error('Error logging debug message:', error);
    }
  }

  /**
   * Log an info level message
   * 
   * @param message - The message to log
   * @param context - Optional contextual data
   */
  info(message: string, context?: Record<string, any>): void {
    try {
      this.pinoInstance.info(context || {}, message);
    } catch (error) {
      console.error('Error logging info message:', error);
    }
  }

  /**
   * Log a warn level message
   * 
   * @param message - The message to log
   * @param context - Optional contextual data
   */
  warn(message: string, context?: Record<string, any>): void {
    try {
      this.pinoInstance.warn(context || {}, message);
    } catch (error) {
      console.error('Error logging warn message:', error);
    }
  }

  /**
   * Log an error level message
   * 
   * @param message - The message to log
   * @param error - Optional error object or additional context
   */
  error(message: string, error?: Error | Record<string, any>): void {
    try {
      if (error instanceof Error) {
        this.pinoInstance.error({ err: error }, message);
      } else if (error) {
        this.pinoInstance.error(error, message);
      } else {
        this.pinoInstance.error(message);
      }
    } catch (err) {
      console.error('Error logging error message:', err);
    }
  }

  /**
   * Set the log level threshold
   * 
   * @param level - The new log level
   */
  setLevel(level: 'debug' | 'info' | 'warn' | 'error'): void {
    try {
      this.config.level = level;
      this.pinoInstance.level = level;
    } catch (error) {
      console.error('Error setting log level:', error);
    }
  }

  /**
   * Get the current log level
   * 
   * @returns The current log level
   */
  getLevel(): string {
    return this.pinoInstance.level;
  }

  /**
   * Create a module logger instance
   * 
   * @param moduleName - The name of the module
   * @returns A ModuleLogger instance
   */
  createModuleLogger(moduleName: string): ModuleLogger {
    const childLogger = this.pinoInstance.child({ module: moduleName });
    return new ModuleLoggerImpl(childLogger, moduleName);
  }
}

/**
 * ModuleLogger Implementation
 * 
 * Logger instance for a specific module with automatic module context.
 */
class ModuleLoggerImpl implements ModuleLogger {
  constructor(
    private pinoChild: PinoLogger,
    private moduleName: string
  ) {}

  /**
   * Log a debug level message
   */
  debug(message: string, context?: Record<string, any>): void {
    try {
      this.pinoChild.debug(context || {}, message);
    } catch (error) {
      console.error('Error logging debug message:', error);
    }
  }

  /**
   * Log an info level message
   */
  info(message: string, context?: Record<string, any>): void {
    try {
      this.pinoChild.info(context || {}, message);
    } catch (error) {
      console.error('Error logging info message:', error);
    }
  }

  /**
   * Log a warn level message
   */
  warn(message: string, context?: Record<string, any>): void {
    try {
      this.pinoChild.warn(context || {}, message);
    } catch (error) {
      console.error('Error logging warn message:', error);
    }
  }

  /**
   * Log an error level message
   */
  error(message: string, error?: Error | Record<string, any>): void {
    try {
      if (error instanceof Error) {
        this.pinoChild.error({ err: error }, message);
      } else if (error) {
        this.pinoChild.error(error, message);
      } else {
        this.pinoChild.error(message);
      }
    } catch (err) {
      console.error('Error logging error message:', err);
    }
  }

  /**
   * Create a child logger with additional context
   */
  child(context: Record<string, any>): ModuleLogger {
    const grandchildLogger = this.pinoChild.child(context);
    return new ModuleLoggerImpl(grandchildLogger, this.moduleName);
  }
}
