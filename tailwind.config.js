/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./index.html", "./src/*.rs", "./src/**/*.rs", "./style/**/*.{html,js,rs}"],
  theme: {
    extend: {
      backgroundColor: {
        'page': '#f9f9f8',
      },
      fontFamily: {
        'custom': ["KinderPeeps"],
      },
      
      // Custom keyframe animations
      keyframes: {
        shine: {
          '0%': { 
            transform: 'translateX(-100%) skewX(-12deg)',
            opacity: '0'
          },
          '50%': { 
            opacity: '1' 
          },
          '100%': { 
            transform: 'translateX(200%) skewX(-12deg)',
            opacity: '0'
          }
        },
        pulse: {
          '0%, 100%': { opacity: '1' },
          '50%': { opacity: '0.5' }
        },
        // New assessment-specific animations
        fadeIn: {
          'from': { opacity: '0' },
          'to': { opacity: '1' }
        },
        slideUp: {
          'from': { 
            opacity: '0',
            transform: 'translateY(20px) scale(0.95)'
          },
          'to': { 
            opacity: '1',
            transform: 'translateY(0) scale(1)'
          }
        },
        slideInUp: {
          'from': {
            opacity: '0',
            transform: 'translateY(30px)'
          },
          'to': {
            opacity: '1',
            transform: 'translateY(0)'
          }
        },
        flow: {
          '0%': { transform: 'translateY(-50%) translateX(-100%)' },
          '50%': { transform: 'translateY(-50%) translateX(0%)' },
          '100%': { transform: 'translateY(-50%) translateX(100%)' }
        },
        wiggle: {
          '0%, 100%': { transform: 'rotate(-3deg)' },
          '50%': { transform: 'rotate(3deg)' }
        },
        bounce: {
          '0%, 100%': { 
            transform: 'translateY(-25%)',
            animationTimingFunction: 'cubic-bezier(0.8, 0, 1, 1)'
          },
          '50%': { 
            transform: 'translateY(0)',
            animationTimingFunction: 'cubic-bezier(0, 0, 0.2, 1)'
          }
        }
      },
      
      // Custom animations
      animation: {
        'shine': 'shine 2s ease-in-out infinite',
        'pulse': 'pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite',
        // New assessment animations
        'fade-in': 'fadeIn 0.3s ease-out',
        'slide-up': 'slideUp 0.3s ease-out',
        'slide-in-up': 'slideInUp 0.4s ease-out both',
        'flow': 'flow 2s ease-in-out infinite',
        'wiggle': 'wiggle 1s ease-in-out infinite',
        'bounce-gentle': 'bounce 1s infinite'
      },
      
      // Custom colors for performance levels
      colors: {
        'performance-high': '#10b981',
        'performance-high-dark': '#059669',
        'performance-medium': '#f59e0b',
        'performance-medium-dark': '#d97706',
        'performance-low': '#ef4444',
        'performance-low-dark': '#dc2626',
        // Assessment-specific colors
        'assessment-primary': '#2e3a59',
        'assessment-primary-dark': '#1e293b',
        'sequence-node': '#3b82f6',
        'sequence-attainment': '#10b981',
        'sequence-optional': '#6b7280',
        'sequence-diagnostic': '#8b5cf6',
        'sequence-remediation': '#f59e0b',
        'sequence-branching': '#eab308'
      },
      
      // Custom gradients
      backgroundImage: {
        'performance-high': 'linear-gradient(135deg, #10b981 0%, #059669 100%)',
        'performance-medium': 'linear-gradient(135deg, #f59e0b 0%, #d97706 100%)',
        'performance-low': 'linear-gradient(135deg, #ef4444 0%, #dc2626 100%)',
        'progress-dark': 'linear-gradient(135deg, #1e293b 0%, #334155 100%)',
        // Assessment-specific gradients
        'assessment-primary': 'linear-gradient(135deg, #2e3a59 0%, #1e293b 100%)',
        'sequence-flow': 'linear-gradient(135deg, #f8fafc 0%, #e2e8f0 100%)',
        'sequence-flow-dark': 'linear-gradient(135deg, #1f2937 0%, #111827 100%)',
        'modal-backdrop': 'linear-gradient(135deg, rgba(0,0,0,0.4) 0%, rgba(0,0,0,0.6) 100%)',
        // Behavior-specific gradients
        'behavior-node': 'linear-gradient(135deg, #3b82f6, #1d4ed8)',
        'behavior-attainment': 'linear-gradient(135deg, #10b981, #047857)',
        'behavior-optional': 'linear-gradient(135deg, #6b7280, #374151)',
        'behavior-diagnostic': 'linear-gradient(135deg, #8b5cf6, #7c3aed)',
        'behavior-remediation': 'linear-gradient(135deg, #f59e0b, #d97706)',
        'behavior-branching': 'linear-gradient(135deg, #eab308, #ca8a04)'
      },
      
      // Custom transitions
      transitionProperty: {
        'width': 'width',
        'height': 'height',
        'spacing': 'margin, padding',
        'all-smooth': 'all'
      },
      
      // Custom transition timing
      transitionTimingFunction: {
        'smooth': 'cubic-bezier(0.4, 0, 0.2, 1)',
        'bounce-in': 'cubic-bezier(0.68, -0.55, 0.265, 1.55)',
        'elastic': 'cubic-bezier(0.68, -0.6, 0.32, 1.6)'
      },
      
      // Custom spacing for assessment components
      spacing: {
        '18': '4.5rem',
        '88': '22rem',
        '100': '25rem',
        '112': '28rem',
        '128': '32rem'
      },
      
      // Custom z-index values
      zIndex: {
        '60': '60',
        '70': '70',
        '80': '80',
        '90': '90',
        '100': '100'
      },
      
      // Custom box shadows
      boxShadow: {
        'assessment': '0 4px 6px -1px rgba(46, 58, 89, 0.1), 0 2px 4px -1px rgba(46, 58, 89, 0.06)',
        'assessment-lg': '0 10px 15px -3px rgba(46, 58, 89, 0.1), 0 4px 6px -2px rgba(46, 58, 89, 0.05)',
        'sequence-item': '0 8px 25px rgba(46, 58, 89, 0.15)',
        'sequence-item-lg': '0 20px 40px rgba(0, 0, 0, 0.3)',
        'modal': '0 25px 50px -12px rgba(0, 0, 0, 0.25)',
        'glow': '0 0 20px rgba(59, 130, 246, 0.3)',
        'glow-green': '0 0 20px rgba(16, 185, 129, 0.3)',
        'glow-orange': '0 0 20px rgba(245, 158, 11, 0.3)'
      },
      
      // Custom border radius
      borderRadius: {
        '4xl': '2rem',
        '5xl': '2.5rem'
      },
      
      // Custom scale values
      scale: {
        '102': '1.02',
        '103': '1.03',
        '115': '1.15'
      }
    },
  },
  plugins: [
    // Your existing scrollbar plugin
    function({ addBase }) {
      addBase({
        '::-webkit-scrollbar': {
          width: '6px',
          height: '6px',
        },
        '::-webkit-scrollbar-track': {
          background: 'transparent',
        },
        '::-webkit-scrollbar-thumb': {
          backgroundColor: 'rgba(203, 213, 225, 0.5)',
          borderRadius: '20px',
        },
        '::-webkit-scrollbar-thumb:hover': {
          backgroundColor: 'rgba(148, 163, 184, 0.7)',
        },
        '*': {
          scrollbarWidth: 'thin',
          scrollbarColor: 'rgba(203, 213, 225, 0.5) transparent',
        },
      });
    },
    
    // Enhanced plugin for custom utilities and animations
    function({ addUtilities, addBase, addComponents }) {
      // Add custom utilities
      const newUtilities = {
        // Progress bar utility
        '.progress-bar': {
          transition: 'width 1s cubic-bezier(0.4, 0, 0.2, 1)'
        },
        
        // Test node hover effects
        '.test-node': {
          transition: 'all 0.3s ease',
          '&:hover': {
            transform: 'scale(1.1)',
            boxShadow: '0 10px 25px rgba(0, 0, 0, 0.2)'
          }
        },
        
        // Performance level backgrounds
        '.performance-high': {
          background: 'linear-gradient(135deg, #10b981 0%, #059669 100%)'
        },
        '.performance-medium': {
          background: 'linear-gradient(135deg, #f59e0b 0%, #d97706 100%)'
        },
        '.performance-low': {
          background: 'linear-gradient(135deg, #ef4444 0%, #dc2626 100%)'
        },
        
        // Custom scrollbar for progress container
        '.progress-container': {
          '&::-webkit-scrollbar': {
            height: '4px'
          },
          '&::-webkit-scrollbar-track': {
            background: '#f1f5f9',
            borderRadius: '2px'
          },
          '&::-webkit-scrollbar-thumb': {
            background: '#cbd5e1',
            borderRadius: '2px',
            '&:hover': {
              background: '#94a3b8'
            }
          }
        },
        
        // Assessment-specific utilities
        '.drag-handle': {
          cursor: 'move',
          '&:hover': {
            opacity: '0.7'
          }
        },
        
        '.glass-effect': {
          background: 'rgba(255, 255, 255, 0.1)',
          backdropFilter: 'blur(10px)',
          border: '1px solid rgba(255, 255, 255, 0.2)'
        },
        
        '.sequence-connector': {
          position: 'relative',
          '&::before': {
            content: '""',
            position: 'absolute',
            top: '50%',
            left: '0',
            right: '0',
            height: '2px',
            background: 'linear-gradient(90deg, transparent, #3b82f6, transparent)',
            transform: 'translateY(-50%)'
          }
        }
      }
      
      // Add responsive and dark mode base styles
      const responsiveStyles = {
        '@media (max-width: 768px)': {
          '.progress-container': {
            padding: '1rem'
          },
          '.test-node': {
            width: '2rem',
            height: '2rem',
            fontSize: '0.75rem'
          },
          '.progress-stats': {
            gridTemplateColumns: '1fr',
            gap: '1rem'
          },
          '.sequence-flow': {
            padding: '1rem'
          },
          '.modal-content': {
            margin: '1rem',
            maxHeight: 'calc(100vh - 2rem)'
          }
        },
        
        '@media (prefers-color-scheme: dark)': {
          '.progress-container': {
            background: 'linear-gradient(135deg, #1e293b 0%, #334155 100%)',
            borderColor: '#475569'
          },
          '.progress-track': {
            background: '#475569'
          },
          '.progress-text': {
            color: '#e2e8f0'
          },
          '.progress-subtext': {
            color: '#94a3b8'
          },
          '.sequence-flow': {
            background: 'linear-gradient(135deg, #1f2937 0%, #111827 100%)',
            borderColor: '#374151'
          }
        },
        
        '@media (prefers-reduced-motion: reduce)': {
          '*': {
            animationDuration: '0.01ms !important',
            animationIterationCount: '1 !important',
            transitionDuration: '0.01ms !important'
          }
        }
      }
      
      // Add component classes
      const newComponents = {
        '.btn-assessment': {
          background: 'linear-gradient(135deg, #2e3a59, #1e293b)',
          color: 'white',
          padding: '0.75rem 1.5rem',
          borderRadius: '0.5rem',
          transition: 'all 0.3s ease',
          position: 'relative',
          overflow: 'hidden',
          '&:hover': {
            transform: 'translateY(-1px)',
            boxShadow: '0 6px 20px rgba(46, 58, 89, 0.4)'
          }
        },
        
        '.sequence-badge': {
          display: 'inline-flex',
          alignItems: 'center',
          justifyContent: 'center',
          width: '2rem',
          height: '2rem',
          borderRadius: '50%',
          fontSize: '0.875rem',
          fontWeight: '600',
          color: 'white',
          background: '#2e3a59',
          position: 'absolute',
          top: '-0.5rem',
          left: '-0.5rem',
          boxShadow: '0 2px 4px rgba(0, 0, 0, 0.1)'
        }
      }
      
      addUtilities(newUtilities)
      addBase(responsiveStyles)
      addComponents(newComponents)
    }
  ],
  
  // Enable dark mode support
  darkMode: 'class', // or 'media' if you prefer system preference
}
