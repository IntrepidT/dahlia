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
        }
      },
      
      // Custom animations
      animation: {
        'shine': 'shine 2s ease-in-out infinite',
        'pulse': 'pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite'
      },
      
      // Custom colors for performance levels
      colors: {
        'performance-high': '#10b981',
        'performance-high-dark': '#059669',
        'performance-medium': '#f59e0b',
        'performance-medium-dark': '#d97706',
        'performance-low': '#ef4444',
        'performance-low-dark': '#dc2626'
      },
      
      // Custom gradients
      backgroundImage: {
        'performance-high': 'linear-gradient(135deg, #10b981 0%, #059669 100%)',
        'performance-medium': 'linear-gradient(135deg, #f59e0b 0%, #d97706 100%)',
        'performance-low': 'linear-gradient(135deg, #ef4444 0%, #dc2626 100%)',
        'progress-dark': 'linear-gradient(135deg, #1e293b 0%, #334155 100%)'
      },
      
      // Custom transitions
      transitionProperty: {
        'width': 'width'
      },
      
      // Custom transition timing
      transitionTimingFunction: {
        'smooth': 'cubic-bezier(0.4, 0, 0.2, 1)'
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
    
    // New plugin for custom utilities and animations
    function({ addUtilities, addBase }) {
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
          }
        }
      }
      
      addUtilities(newUtilities)
      addBase(responsiveStyles)
    }
  ],
  
  // Enable dark mode support
  darkMode: 'class', // or 'media' if you prefer system preference
}
