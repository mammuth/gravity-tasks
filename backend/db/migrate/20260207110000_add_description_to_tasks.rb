class AddDescriptionToTasks < ActiveRecord::Migration[8.1]
  def change
    add_column :tasks, :description, :text
  end
end
