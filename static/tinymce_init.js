import tinymce from 'tinymce';

window.initializeTinyMCE = function(elementId) {
  tinymce.init({
    selector: '#${elementId}',
    plugins: 'lists link image table code',
    toolbar: 'undo redo | bold italic | alignleft aligncenter alignright | numlist bullist outdent indent | link image',
    menubar: false,
  });
};
