import os
import jinja2
import json
import shutil

config_file_path = "template-input.json"
template_path = "template"
destination_path = "project"

config_file = open(config_file_path, 'r')
config = json.load(config_file)

current_dir = os.getcwd().replace("\\","/")
env = jinja2.Environment(loader=jinja2.FileSystemLoader(current_dir))

# Iterate over each file in template directory
for root, dirs, files in os.walk(template_path):
    for file in files:
        
        # Render template
        file_relative_path = os.path.join(root,file).replace("\\","/")
        print("File: " + file_relative_path)
        
        # Determine output path
        relative_path = os.path.relpath(root, template_path).replace("\\","/")
        print("Relative path: " + relative_path)
        output_path = os.path.join(destination_path, relative_path).replace("\\","/")
        print("Output path: " + output_path)
        os.makedirs(output_path, exist_ok=True)
        output_file_path = os.path.join(output_path, file).replace("\\","/")
        
        if file.endswith(('.yaml', '.env', '.json', '.toml', '.properties', 'Dockerfile')):
            print("Template file: " + file_relative_path + " to " + output_file_path)
            template = env.get_template(file_relative_path)
            rendered_content = template.render(config)
            
            # Write rendered content to output file
            with open(output_file_path, 'w') as f:
                f.write(rendered_content)
        else:
            print("Copy file: " + file_relative_path + " to " + output_file_path)
            shutil.copy2(file_relative_path, output_file_path)